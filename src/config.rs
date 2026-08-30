use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::PathBuf,
    sync::{LazyLock, OnceLock, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Config {
    pub scale_x: u16,      // 1-8
    pub scale_y: u16,      // 1-5
    pub board_width: u16,  // 4-50
    pub board_height: u16, // 4-50
    pub start_level: u16,  // 0-30
}

impl Default for Config {
    fn default() -> Self {
        Self {
            scale_x: 4,
            scale_y: 2,
            board_width: 10,
            board_height: 20,
            start_level: 1,
        }
    }
}

static CONFIG: LazyLock<RwLock<Config>> = LazyLock::new(|| RwLock::new(Config::load()));
static DEFAULT_CONFIG: OnceLock<Config> = OnceLock::new();

pub fn config() -> RwLockReadGuard<'static, Config> {
    CONFIG.read().expect("config lock poisoned")
}

pub fn config_mut() -> RwLockWriteGuard<'static, Config> {
    CONFIG.write().expect("config lock poisoned")
}

pub fn default_config() -> &'static Config {
    DEFAULT_CONFIG.get_or_init(Config::default)
}

impl Config {
    fn normalized(mut self) -> Self {
        self.scale_x = self.scale_x.clamp(1, 8);
        self.scale_y = self.scale_y.clamp(1, 5);
        self.board_width = self.board_width.clamp(4, 50);
        self.board_height = self.board_height.clamp(4, 50);
        self.start_level = self.start_level.clamp(0, 30);
        self
    }

    fn path() -> Option<PathBuf> {
        dirs::data_dir().map(|path| {
            path.join("dev.orangishcat.cursed-tetris")
                .join("config.json")
        })
    }

    pub fn load() -> Config {
        let Some(path) = Self::path() else {
            return Self::default();
        };
        let Ok(contents) = fs::read_to_string(path) else {
            return Self::default();
        };
        serde_json::from_str::<Self>(&contents)
            .unwrap_or_default()
            .normalized()
    }

    pub fn save(&self) {
        let Some(path) = Self::path() else { return };
        let Some(parent) = path.parent() else { return };
        if fs::create_dir_all(parent).is_err() {
            return;
        }
        if let Ok(contents) = serde_json::to_string_pretty(self) {
            let _ = fs::write(path, contents);
        }
    }
}
