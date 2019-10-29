use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct GeneralError{ msg: String }

impl GeneralError {
    pub fn new(msg: String) -> GeneralError {
        GeneralError { msg }
    }
}

impl fmt::Display for GeneralError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl Error for GeneralError {}