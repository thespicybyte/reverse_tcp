use thiserror::Error;

#[derive(Debug, Error)]
pub enum ServerError {
    #[error("invalid port number: '{0}'")]
    InvalidPort(String),

    #[error("invalid listen address: '{0}'")]
    InvalidIpAddress(String),

    #[error("error starting listener: {0}")]
    ListenError(#[from] std::io::Error),
}

// let err = ServerError::InvalidPort("fvgfdv3v".to_string());
// error!(error = %err, "server error");
