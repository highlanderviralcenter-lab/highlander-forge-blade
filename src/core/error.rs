//! Erros do core — hierarquia unificada

#[derive(Debug, Clone, thiserror::Error)]
pub enum CoreError {
    #[error("WMI indisponivel: {0}")]
    WmiUnavailable(String),
    #[error("Registry inacessivel: {0}")]
    RegistryAccess(String),
    #[error("Servico nao encontrado: {0}")]
    ServiceNotFound(String),
    #[error("Permissao negada: {0}")]
    PermissionDenied(String),
    #[error("Operacao nao suportada: {0}")]
    NotSupported(String),
    #[error("Timeout: {0}")]
    Timeout(String),
    #[error("Erro de IO: {0}")]
    Io(String),
    #[error("Erro de serializacao: {0}")]
    Serialization(String),
    #[error("Estado inconsistente: {0}")]
    InvalidState(String),
    #[error("Operacao abortada pelo usuario")]
    UserCancelled,
}

impl From<std::io::Error> for CoreError {
    fn from(e: std::io::Error) -> Self {
        CoreError::Io(e.to_string())
    }
}
