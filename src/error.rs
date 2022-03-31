use reqwest::Error as HttpError;
use std::{
    fmt::{self, Display},
    io::Error as IoError,
};
use trust_dns_proto::error::ProtoError;

#[derive(Debug)]
pub enum UpstreamError {
    Io(IoError),
    Proto(ProtoError),
    Http(HttpError),
}

impl Display for UpstreamError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            UpstreamError::Io(ref cause) => {
                write!(f, "[upstream] {}", cause)
            }
            UpstreamError::Proto(ref cause) => {
                write!(f, "[upstream] {}", cause)
            }
            UpstreamError::Http(ref cause) => {
                write!(f, "[upstream] {}", cause)
            }
        }
    }
}

impl From<IoError> for UpstreamError {
    fn from(cause: IoError) -> UpstreamError {
        UpstreamError::Io(cause)
    }
}

impl From<ProtoError> for UpstreamError {
    fn from(cause: ProtoError) -> UpstreamError {
        UpstreamError::Proto(cause)
    }
}

impl From<HttpError> for UpstreamError {
    fn from(cause: HttpError) -> UpstreamError {
        UpstreamError::Http(cause)
    }
}
