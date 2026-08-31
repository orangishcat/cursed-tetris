use std::{
    path::Path,
    sync::{LazyLock, Mutex, RwLock, RwLockReadGuard, RwLockWriteGuard},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use rusqlite::{Connection, OptionalExtension, Transaction, TransactionBehavior, params};
use rustrict::CensorStr;

pub const LEADERBOARD_LIMIT: usize = 50;
pub const USERNAME_MAX_LEN: usize = 12;

static ONLINE: LazyLock<RwLock<Option<Online>>> = LazyLock::new(|| RwLock::new(None));

pub fn online() -> RwLockReadGuard<'static, Option<Online>> {
    ONLINE.read().expect("online lock poisoned")
}

pub fn online_mut() -> RwLockWriteGuard<'static, Option<Online>> {
    ONLINE.write().expect("online lock poisoned")
}

pub fn set_online(value: Option<Online>) {
    *online_mut() = value;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LeaderboardEntry {
    pub rank: usize,
    pub username: String,
    pub score: u32,
    pub is_current: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Qualification {
    pub rank: usize,
    pub username: Option<String>,
}

pub struct Online {
    connection: Mutex<Connection>,
    id: String,
    entries: Vec<LeaderboardEntry>,
    load_error: Option<String>,
}

impl Online {
    pub fn open(path: &Path, id: String) -> rusqlite::Result<Self> {
        let connection = Connection::open(path)?;
        connection.busy_timeout(Duration::from_secs(5))?;
        connection.pragma_update(None, "journal_mode", "WAL")?;
        connection.pragma_update(None, "foreign_keys", true)?;
        connection.execute_batch(
            "CREATE TABLE IF NOT EXISTS players (
                id         TEXT PRIMARY KEY,
                username   TEXT NOT NULL UNIQUE COLLATE NOCASE,
                created_at INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS scores (
                player_id  TEXT PRIMARY KEY REFERENCES players(id),
                score      INTEGER NOT NULL CHECK(score >= 0),
                achieved_at INTEGER NOT NULL
            );
            CREATE INDEX IF NOT EXISTS scores_ranking
                ON scores(score DESC, achieved_at ASC, player_id ASC);",
        )?;

        let mut online = Self {
            connection: Mutex::new(connection),
            id,
            entries: Vec::new(),
            load_error: None,
        };
        online.refresh();
        Ok(online)
    }

    pub fn entries(&self) -> &[LeaderboardEntry] {
        &self.entries
    }

    pub fn load_error(&self) -> Option<&str> {
        self.load_error.as_deref()
    }

    pub fn refresh(&mut self) {
        let connection = self
            .connection
            .lock()
            .expect("online database lock poisoned");
        match Self::load_entries(&connection, &self.id) {
            Ok(entries) => {
                self.entries = entries;
                self.load_error = None;
            }
            Err(error) => self.load_error = Some(error.to_string()),
        }
    }

    pub fn qualification(&self, score: u32) -> Result<Option<Qualification>, String> {
        if score < 50 {
            return Ok(None);
        }
        let connection = self
            .connection
            .lock()
            .expect("online database lock poisoned");
        let best = connection
            .query_row(
                "SELECT score FROM scores WHERE player_id = ?1",
                [&self.id],
                |row| row.get::<_, u32>(0),
            )
            .optional()
            .map_err(|error| error.to_string())?;
        if best.is_some_and(|best| score <= best) {
            return Ok(None);
        }

        let rank =
            candidate_rank(&connection, &self.id, score).map_err(|error| error.to_string())?;
        if rank > LEADERBOARD_LIMIT {
            return Ok(None);
        }

        let username = connection
            .query_row(
                "SELECT username FROM players WHERE id = ?1",
                [&self.id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|error| error.to_string())?;
        Ok(Some(Qualification { rank, username }))
    }

    pub fn submit(&mut self, score: u32, username: Option<&str>) -> Result<usize, String> {
        if let Some(username) = username {
            validate_username(username)?;
        }

        let id = self.id.clone();
        let mut connection = self
            .connection
            .lock()
            .expect("online database lock poisoned");
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|error| error.to_string())?;
        let rank = submit_in_transaction(&transaction, &id, score, username)?;
        transaction.commit().map_err(|error| error.to_string())?;
        drop(connection);
        self.refresh();
        Ok(rank)
    }

    fn load_entries(connection: &Connection, id: &str) -> rusqlite::Result<Vec<LeaderboardEntry>> {
        let mut statement = connection.prepare(
            "SELECT players.id, players.username, scores.score
             FROM scores
             JOIN players ON players.id = scores.player_id
             ORDER BY scores.score DESC, scores.achieved_at ASC, scores.player_id ASC
             LIMIT 50",
        )?;
        statement
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, u32>(2)?,
                ))
            })?
            .enumerate()
            .map(|(index, row)| {
                let (player_id, username, score) = row?;
                Ok(LeaderboardEntry {
                    rank: index + 1,
                    username,
                    score,
                    is_current: player_id == id,
                })
            })
            .collect()
    }
}

fn candidate_rank(connection: &Connection, id: &str, score: u32) -> rusqlite::Result<usize> {
    let rank = connection.query_row(
        "SELECT 1 + COUNT(*) FROM scores WHERE player_id <> ?1 AND score >= ?2",
        params![id, score],
        |row| row.get::<_, i64>(0),
    )?;
    Ok(rank.try_into().expect("SQLite COUNT result fits usize"))
}

fn submit_in_transaction(
    transaction: &Transaction<'_>,
    id: &str,
    score: u32,
    username: Option<&str>,
) -> Result<usize, String> {
    let best = transaction
        .query_row(
            "SELECT score FROM scores WHERE player_id = ?1",
            [id],
            |row| row.get::<_, u32>(0),
        )
        .optional()
        .map_err(|error| error.to_string())?;
    if best.is_some_and(|best| score <= best) {
        return Err("This score is not a new personal best.".into());
    }

    let rank = candidate_rank(transaction, id, score).map_err(|error| error.to_string())?;
    if rank > LEADERBOARD_LIMIT {
        return Err("This score no longer qualifies for the top 50.".into());
    }

    let existing_username = transaction
        .query_row("SELECT username FROM players WHERE id = ?1", [id], |row| {
            row.get::<_, String>(0)
        })
        .optional()
        .map_err(|error| error.to_string())?;
    if existing_username.is_none() {
        let username = username.ok_or("Enter a username before submitting.")?;
        transaction
            .execute(
                "INSERT INTO players(id, username, created_at) VALUES (?1, ?2, ?3)",
                params![id, username, now_millis()],
            )
            .map_err(|error| {
                if error
                    .to_string()
                    .contains("UNIQUE constraint failed: players.username")
                {
                    "That username is already taken.".into()
                } else {
                    error.to_string()
                }
            })?;
    }

    let latest_achievement = transaction
        .query_row(
            "SELECT COALESCE(MAX(achieved_at), 0) FROM scores",
            [],
            |row| row.get::<_, i64>(0),
        )
        .map_err(|error| error.to_string())?;
    let achieved_at = now_millis().max(latest_achievement.saturating_add(1));
    transaction
        .execute(
            "INSERT INTO scores(player_id, score, achieved_at) VALUES (?1, ?2, ?3)
             ON CONFLICT(player_id) DO UPDATE SET
                 score = excluded.score,
                 achieved_at = excluded.achieved_at
             WHERE excluded.score > scores.score",
            params![id, score, achieved_at],
        )
        .map_err(|error| error.to_string())?;
    transaction
        .execute(
            "DELETE FROM scores WHERE player_id IN (
                SELECT player_id FROM scores
                ORDER BY score DESC, achieved_at ASC, player_id ASC
                LIMIT -1 OFFSET 50
            )",
            [],
        )
        .map_err(|error| error.to_string())?;
    Ok(rank)
}

pub fn validate_username(username: &str) -> Result<(), String> {
    if username.is_empty() || username.len() > USERNAME_MAX_LEN {
        return Err(format!("Username must be 1-{USERNAME_MAX_LEN} characters."));
    }
    if !username
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
    {
        return Err("Use only letters, numbers, underscores, or hyphens.".into());
    }
    if username.is_inappropriate() {
        return Err(
            "Username might be inappropriate, please choose a different name. Sorry :(".into(),
        );
    }
    Ok(())
}

fn now_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .try_into()
        .unwrap_or(i64::MAX)
}
