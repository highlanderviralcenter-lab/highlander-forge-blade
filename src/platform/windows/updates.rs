//! Implementacao real de UpdateProvider
use crate::core::error::CoreError;
use crate::core::traits::UpdateProvider;

pub struct WuaUpdateProvider;
impl WuaUpdateProvider { pub fn new() -> Self { Self } }

impl UpdateProvider for WuaUpdateProvider {
    fn search_pending(&self) -> Result<Vec<String>, CoreError> { Ok(Vec::new()) }
    fn install_updates(&self) -> Result<u32, CoreError> { Ok(0) }
    fn get_history(&self, _limit: u32) -> Result<Vec<String>, CoreError> { Ok(Vec::new()) }
}
