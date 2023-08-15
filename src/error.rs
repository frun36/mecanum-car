use rppal::gpio;
use std::{
    fmt::{Display, Formatter, Result},
    io,
    sync,
};

#[derive(Debug)]
pub enum Error {
    Gpio(gpio::Error),
    Io(io::Error),
    ActixWeb(actix_web::Error),
    PoisonError(String),
    SendError(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::Gpio(error) => write!(f, "GPIO error: {error}"),
            Self::Io(error) => write!(f, "IO error: {error}"),
            Self::ActixWeb(error) => write!(f, "Actix Web error: {error}"),
            Self::PoisonError(error) => write!(f, "Poison error: {error}"),
            Self::SendError(error) => write!(f, "Send error: {error}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<gpio::Error> for Error {
    fn from(error: gpio::Error) -> Self {
        Self::Gpio(error)
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<actix_web::Error> for Error {
    fn from(error: actix_web::Error) -> Self {
        Self::ActixWeb(error)
    }
}

impl<T> From<sync::PoisonError<T>> for Error {
    fn from(error: sync::PoisonError<T>) -> Self {
        Self::PoisonError(format!("{error}"))
    }
}

impl<T> From<actix::prelude::SendError<T>> for Error {
    fn from(error: actix::prelude::SendError<T>) -> Self {
        Self::SendError(format!("{error}"))
    }
}

// Not sure if it's safe
unsafe impl Send for Error {}