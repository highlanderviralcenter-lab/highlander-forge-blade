//! Configuracao externa TOML/JSON com resolucao correta de caminhos.
//!
//! Prioridade de resolucao de `base_dir`:
//! 1. Variavel de ambiente `HFB_BASE_DIR`
//! 2. Arquivo de config (`hfb.toml` / `hfb.json`) em: dir atual,
//!    %PROGRAMDATA%\HighlanderForgeBlade, dir do executavel
//! 3. Default: `%PROGRAMDATA%\HighlanderForgeBlade` (Windows) ou
//!    `~/.local/share/highlander-forge-blade` (demais plataformas p/ testes)

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub const APP_DIR_NAME: &str = "HighlanderForgeBlade";
pub const STATE_FILE: &str = "estado_manutencao.json";
pub const MACHINE_ID_FILE: &str = "machine_id";
pub const LOGS_DIR: &str = "Logs";
pub const REPORTS_DIR: &str = "Reports";
pub const POST_REBOOT_FLAG: &str = "post_reboot_pending.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Diretorio base de dados/logs/estado. Vazio => resolvido automaticamente.
    pub base_dir: String,
    pub log_level: String,
    pub auto_update: bool,
    /// Idade minima (dias) para exclusao de arquivos temporarios — seguranca P0.
    pub temp_min_age_days: u64,
    /// Exclusao de cache de navegadores desligada por padrao (risco de dados de sessao).
    pub clean_browser_cache: bool,
    /// Limpeza da lixeira em TODOS os volumes fixos (nao apenas C:).
    pub clean_recycle_bin_all_volumes: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            base_dir: String::new(),
            log_level: "info".to_string(),
            auto_update: true,
            temp_min_age_days: 3,
            clean_browser_cache: false,
            clean_recycle_bin_all_volumes: true,
        }
    }
}

impl Config {
    /// Resolve o diretorio base efetivo (CORRIGIDO: sem `C:\\\\` duplicado).
    pub fn resolved_base_dir(&self) -> PathBuf {
        if !self.base_dir.trim().is_empty() {
            return PathBuf::from(self.base_dir.trim());
        }
        if let Ok(env_dir) = std::env::var("HFB_BASE_DIR") {
            if !env_dir.trim().is_empty() {
                return PathBuf::from(env_dir.trim());
            }
        }
        #[cfg(windows)]
        {
            let pd = std::env::var("PROGRAMDATA")
                .unwrap_or_else(|_| r"C:\ProgramData".to_string());
            PathBuf::from(pd).join(APP_DIR_NAME)
        }
        #[cfg(not(windows))]
        {
            dirs::data_local_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(APP_DIR_NAME)
        }
    }

    pub fn state_path(&self) -> PathBuf {
        self.resolved_base_dir().join(STATE_FILE)
    }
    pub fn machine_id_path(&self) -> PathBuf {
        self.resolved_base_dir().join(MACHINE_ID_FILE)
    }
    pub fn logs_dir(&self) -> PathBuf {
        self.resolved_base_dir().join(LOGS_DIR)
    }
    pub fn reports_dir(&self) -> PathBuf {
        self.resolved_base_dir().join(REPORTS_DIR)
    }
    pub fn post_reboot_flag_path(&self) -> PathBuf {
        self.resolved_base_dir().join(POST_REBOOT_FLAG)
    }

    /// Garante que a estrutura de diretorios exista.
    pub fn ensure_dirs(&self) -> std::io::Result<()> {
        std::fs::create_dir_all(self.resolved_base_dir())?;
        std::fs::create_dir_all(self.logs_dir())?;
        std::fs::create_dir_all(self.reports_dir())?;
        Ok(())
    }

    /// Carrega config de disco (TOML ou JSON) respeitando prioridade env > arquivo > default.
    pub fn load() -> Config {
        let mut cfg = Config::default();

        // 1. Arquivos de config procurados em ordem.
        let candidates = config_candidates();
        for path in candidates {
            match try_load_file(&path) {
                Ok(Some(mut file_cfg)) => {
                    // Env ainda vence sobre o arquivo.
                    if let Ok(env_dir) = std::env::var("HFB_BASE_DIR") {
                        file_cfg.base_dir = env_dir;
                    }
                    return file_cfg;
                }
                Ok(None) => {}
                Err(e) => {
                    tracing::warn!("Ignorando config invalido {}: {}", path.display(), e);
                }
            }
        }

        // 2. Env direto.
        if let Ok(env_dir) = std::env::var("HFB_BASE_DIR") {
            cfg.base_dir = env_dir;
        }
        cfg
    }

    /// Escreve um template de configuracao no diretorio base (para deploy documentado).
    pub fn write_template_if_missing(&self) -> std::io::Result<Option<PathBuf>> {
        let path = self.resolved_base_dir().join("hfb.toml");
        if path.exists() {
            return Ok(None);
        }
        std::fs::create_dir_all(self.resolved_base_dir())?;
        let toml = "# Highlander Forge Blade — configuracao\nbase_dir = \"\"                # vazio = %PROGRAMDATA%\\HighlanderForgeBlade\nlog_level = \"info\"\nauto_update = true\ntemp_min_age_days = 3\nclean_browser_cache = false\nclean_recycle_bin_all_volumes = true\n";
        std::fs::write(&path, toml)?;
        Ok(Some(path))
    }
}

fn config_candidates() -> Vec<PathBuf> {
    let mut v = Vec::new();
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            v.push(dir.join("hfb.toml"));
            v.push(dir.join("hfb.json"));
        }
    }
    v.push(PathBuf::from("hfb.toml"));
    v.push(PathBuf::from("hfb.json"));
    #[cfg(windows)]
    {
        let pd = std::env::var("PROGRAMDATA").unwrap_or_else(|_| r"C:\ProgramData".to_string());
        v.push(Path::new(&pd).join(APP_DIR_NAME).join("hfb.toml"));
    }
    #[cfg(not(windows))]
    {
        if let Some(d) = dirs::config_dir() {
            v.push(d.join(APP_DIR_NAME).join("hfb.toml"));
        }
    }
    v
}

fn try_load_file(path: &Path) -> Result<Option<Config>, String> {
    if !path.is_file() {
        return Ok(None);
    }
    let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let cfg: Config = if path.extension().and_then(|s| s.to_str()) == Some("json") {
        serde_json::from_str(&content).map_err(|e| e.to_string())?
    } else {
        toml::from_str(&content).map_err(|e| e.to_string())?
    };
    Ok(Some(cfg))
}

/// Compatibilidade retroativa (usado por chamadas antigas).
pub fn load_or_default() -> Config {
    Config::load()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_paths_are_well_formed() {
        let cfg = Config::default();
        let base = cfg.resolved_base_dir();
        let s = base.to_string_lossy().replace('\\', "/");
        // Bug antigo: r"C:\\ManutencaoWindows" gerava caminho com barra dupla.
        assert!(!s.contains("//"), "caminho com barra duplicada: {}", s);
        assert!(s.ends_with(APP_DIR_NAME), "caminho inesperado: {}", s);
        assert!(cfg.state_path().ends_with(STATE_FILE));
        assert!(cfg.machine_id_path().ends_with(MACHINE_ID_FILE));
    }

    #[test]
    fn explicit_base_dir_wins() {
        let cfg = Config { base_dir: "/tmp/hfb-test".into(), ..Default::default() };
        assert_eq!(cfg.resolved_base_dir(), PathBuf::from("/tmp/hfb-test"));
    }

    #[test]
    fn toml_roundtrip() {
        let cfg = Config { base_dir: "D:\\HFB".into(), log_level: "debug".into(),
            auto_update: false, temp_min_age_days: 7,
            clean_browser_cache: true, clean_recycle_bin_all_volumes: false };
        let toml_s = toml::to_string(&cfg).unwrap();
        let back: Config = toml::from_str(&toml_s).unwrap();
        assert_eq!(back.temp_min_age_days, 7);
        assert!(!back.auto_update);
    }
}
