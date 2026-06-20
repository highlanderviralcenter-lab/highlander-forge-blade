//! Implementacao real de CleanupProvider
use crate::core::error::CoreError;
use crate::core::traits::CleanupProvider;
use std::process::Command;
use std::path::Path;

pub struct WinCleanupProvider;

impl WinCleanupProvider {
    pub fn new() -> Self {
        Self
    }
}

impl CleanupProvider for WinCleanupProvider {
    fn clean_temp_files(&self) -> Result<u64, CoreError> {
        let temp_dir = std::env::temp_dir();
        let mut bytes_freed: u64 = 0;

        if let Ok(entries) = std::fs::read_dir(&temp_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_file() {
                        bytes_freed += metadata.len();
                        let _ = std::fs::remove_file(&path);
                    } else if metadata.is_dir() {
                        let _ = std::fs::remove_dir_all(&path);
                    }
                }
            }
        }

        Ok(bytes_freed)
    }

    fn clean_recycle_bin(&self) -> Result<u64, CoreError> {
        // CORRECAO: raw string r"..." para evitar escape de \ e $
        let _output = Command::new("cmd")
            .args(["/c", r"rd /s /q C:\$Recycle.Bin 2>nul"])
            .output()
            .map_err(|e| CoreError::Io(e.to_string()))?;

        Ok(0)
    }

    fn clean_browser_cache(&self) -> Result<u64, CoreError> {
        let local_app_data = std::env::var("LOCALAPPDATA")
            .unwrap_or_else(|_| String::from(r"C:\Users\Default\AppData\Local"));

        let mut bytes_freed: u64 = 0;

        let chrome_cache = Path::new(&local_app_data).join(r"Google\Chrome\User Data\Default\Cache");
        if chrome_cache.exists() {
            if let Ok(entries) = std::fs::read_dir(&chrome_cache) {
                for entry in entries.flatten() {
                    if let Ok(metadata) = entry.metadata() {
                        if metadata.is_file() {
                            bytes_freed += metadata.len();
                            let _ = std::fs::remove_file(entry.path());
                        }
                    }
                }
            }
        }

        let edge_cache = Path::new(&local_app_data).join(r"Microsoft\Edge\User Data\Default\Cache");
        if edge_cache.exists() {
            if let Ok(entries) = std::fs::read_dir(&edge_cache) {
                for entry in entries.flatten() {
                    if let Ok(metadata) = entry.metadata() {
                        if metadata.is_file() {
                            bytes_freed += metadata.len();
                            let _ = std::fs::remove_file(entry.path());
                        }
                    }
                }
            }
        }

        Ok(bytes_freed)
    }

    fn run_dism(&self) -> Result<(), CoreError> {
        let _output = Command::new("dism")
            .args(["/Online", "/Cleanup-Image", "/RestoreHealth"])
            .output()
            .map_err(|e| CoreError::Io(e.to_string()))?;

        Ok(())
    }

    fn run_sfc(&self) -> Result<(), CoreError> {
        let _output = Command::new("sfc")
            .args(["/scannow"])
            .output()
            .map_err(|e| CoreError::Io(e.to_string()))?;

        Ok(())
    }
}
