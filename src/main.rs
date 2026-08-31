mod app;
mod config;
mod leaderboard;
mod piece;
mod powerup;
mod screen;
mod state;
mod task;
use std::{env, error::Error, ffi::OsString, path::PathBuf, process};

use config::set_persistence_enabled;
use leaderboard::{Online, set_online};

const USAGE: &str = "Usage: cursed-tetris [--online DATABASE --id PLAYER_ID]";

#[derive(Debug, Default, PartialEq, Eq)]
struct Args {
    online: Option<PathBuf>,
    id: Option<String>,
    help: bool,
}

impl Args {
    fn parse(arguments: impl IntoIterator<Item = OsString>) -> Result<Self, String> {
        let mut parsed = Self::default();
        let mut arguments = arguments.into_iter();
        while let Some(argument) = arguments.next() {
            match argument.to_str() {
                Some("--online") if parsed.online.is_none() => {
                    parsed.online = Some(
                        arguments
                            .next()
                            .filter(|value| !value.is_empty())
                            .map(PathBuf::from)
                            .ok_or("--online requires a database path")?,
                    );
                }
                Some("--id") if parsed.id.is_none() => {
                    let id = arguments
                        .next()
                        .and_then(|value| value.into_string().ok())
                        .ok_or("--id requires a UTF-8 player ID")?;
                    if !valid_id(&id) {
                        return Err(
                            "--id must be 1-128 letters, numbers, '.', '_', ':', or '-'".into()
                        );
                    }
                    parsed.id = Some(id);
                }
                Some("--help" | "-h") => parsed.help = true,
                Some("--online") => return Err("--online may only be provided once".into()),
                Some("--id") => return Err("--id may only be provided once".into()),
                Some(argument) => return Err(format!("unknown argument: {argument}")),
                None => return Err("arguments must be valid UTF-8".into()),
            }
        }
        if parsed.online.is_some() != parsed.id.is_some() {
            return Err("--online and --id must be provided together".into());
        }
        Ok(parsed)
    }
}

fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 128
        && id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b':' | b'-'))
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse(env::args_os().skip(1)).unwrap_or_else(|error| {
        eprintln!("{error}\n{USAGE}");
        process::exit(2);
    });
    if args.help {
        println!("{USAGE}");
        return Ok(());
    }
    set_persistence_enabled(args.online.is_none());
    let online = match (args.online, args.id) {
        (Some(path), Some(id)) => Some(Online::open(&path, id)?),
        (None, None) => None,
        _ => unreachable!("argument validation requires both online arguments"),
    };
    set_online(online);
    ratatui::run(|terminal| app::App::default().run(terminal))?;
    Ok(())
}
