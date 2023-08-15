use rppal::gpio;
use std::{
    fmt::{Display, Formatter, Result},
    io, sync,
};

#[derive(Debug)]
pub struct Error {
    msg: String,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.msg)
    }
}

impl std::error::Error for Error {}

impl From<gpio::Error> for Error {
    fn from(value: gpio::Error) -> Self {
        Self {
            msg: value.to_string(),
        }
    }
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self {
            msg: value.to_string(),
        }
    }
}

impl From<actix_web::Error> for Error {
    fn from(value: actix_web::Error) -> Self {
        Self {
            msg: value.to_string(),
        }
    }
}

impl<T> From<sync::PoisonError<T>> for Error {
    fn from(value: sync::PoisonError<T>) -> Self {
        Self {
            msg: value.to_string(),
        }
    }
}

impl<T> From<actix::prelude::SendError<T>> for Error {
    fn from(value: actix::prelude::SendError<T>) -> Self {
        Self {
            msg: value.to_string(),
        }
    }
}
