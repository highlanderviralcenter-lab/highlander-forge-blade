//! Configuracao externa TOML/JSON

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Config {
    pub base_dir: String,
    pub log_level: String,
    pub auto_update: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            base_dir: r"C:\ManutencaoWindows".to_string(),
            log_level: "info".to_string(),
            auto_update: true,
        }
    }
}

pub fn load_or_default() -> Config {
    Config::default()
}
