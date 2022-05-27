use thiserror::Error;

#[derive(Error, Debug)]
pub enum LocalError {
    #[error("failed to parse the address {0}:{1}")]
    InvalidAddress(String, u16),

    #[error("failed to bind to the address {0}:{1} (Permission denied)")]
    PermissionDenied(String, u16),

    #[error("failed to bind to the address {0}:{1}")]
    Unknown(String, u16),
}

#[derive(Error, Debug)]
pub enum UpstreamError {
    #[error("failed to build the HTTPS client")]
    Build,

    #[error("failed to bootstrap the address {0}: {1}")]
    Bootstrap(String, String),

    #[error("failed to resolve the DNS request")]
    Resolve,
}
