use crossbeam_channel::{RecvError, SendError};
use md5::Digest;
use std::fmt::{Display, Formatter};
use std::io;

#[derive(Debug)]
pub enum ThesisError {
    CreateError(String),
    MergeError(String),
    IOError(io::Error),
    ChannelError(ChannelErrorEnum),
}

#[derive(Debug)]
pub enum ChannelErrorEnum {
    RecvError(RecvError),
    SendError(SendError<Option<Digest>>),
}

pub type ThesisResult<T> = Result<T, ThesisError>;

impl Display for ThesisError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ThesisError::CreateError(s) => write!(f, "{}", s),
            ThesisError::MergeError(s) => write!(f, "{}", s),
            ThesisError::IOError(e) => write!(f, "{}", e),
            ThesisError::ChannelError(ChannelErrorEnum::RecvError(e)) => write!(f, "{}", e),
            ThesisError::ChannelError(ChannelErrorEnum::SendError(e)) => write!(f, "{}", e),
        }
    }
}

impl From<io::Error> for ThesisError {
    fn from(err: io::Error) -> Self {
        ThesisError::IOError(err)
    }
}

impl From<RecvError> for ThesisError {
    fn from(err: RecvError) -> Self {
        ThesisError::ChannelError(ChannelErrorEnum::RecvError(err))
    }
}

impl From<SendError<Option<Digest>>> for ThesisError {
    fn from(err: SendError<Option<Digest>>) -> Self {
        ThesisError::ChannelError(ChannelErrorEnum::SendError(err))
    }
}
