//! Engine de manutencao — codigo puro, sem dependencia de UI
//!
//! DT-09: Todos os componentes Windows-specific usam traits para testabilidade.
//! Implementacoes reais em platform/windows/.
//! Mocks gerados automaticamente com mockall em testes.

pub mod error;
pub mod traits;
pub mod audit;
pub mod cleanup;
pub mod registry;
pub mod services;
pub mod updates;
pub mod repair;
pub mod security;
pub mod system;
pub mod storage;
pub mod report;
