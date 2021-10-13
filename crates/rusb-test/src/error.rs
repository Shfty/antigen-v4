#[derive(Debug)]
pub enum Error {
    Application(String),
    Rusb(rusb::Error),
    Io(std::io::Error),
    Fmt(std::fmt::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Application(e) => f.write_str(e),
            Error::Rusb(e) => e.fmt(f),
            Error::Io(e) => e.fmt(f),
            Error::Fmt(e) => e.fmt(f),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Application(_) => None,
            Error::Rusb(e) => Some(e),
            Error::Io(e) => Some(e),
            Error::Fmt(e) => Some(e),
        }
    }
}

impl From<&str> for Error {
    fn from(s: &str) -> Self {
        Error::Application(s.into())
    }
}

impl From<rusb::Error> for Error {
    fn from(e: rusb::Error) -> Self {
        Error::Rusb(e)
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

impl From<std::fmt::Error> for Error {
    fn from(e: std::fmt::Error) -> Self {
        Error::Fmt(e)
    }
}

