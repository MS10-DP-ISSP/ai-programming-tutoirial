use std::fmt;

/// All failures the crate can report.
#[derive(Debug)]
pub enum Error {
    /// Invalid value given to a constructor or the command line.
    InvalidInput(String),
    /// `World::random` could not place all balls without overlap.
    PlacementFailed,
    /// A GIF was requested without any frames.
    NoFrames,
    /// A frame does not match the GIF's declared size.
    FrameSizeMismatch(String),
    /// File-system or stream failure.
    Io(std::io::Error),
    /// GIF encoder failure.
    Gif(gif::EncodingError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::InvalidInput(msg) => write!(f, "invalid input: {msg}"),
            Error::PlacementFailed => write!(f, "could not place balls without overlap"),
            Error::NoFrames => write!(f, "cannot write a GIF without frames"),
            Error::FrameSizeMismatch(msg) => write!(f, "frame size mismatch: {msg}"),
            Error::Io(e) => write!(f, "I/O error: {e}"),
            Error::Gif(e) => write!(f, "GIF error: {e}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io(e) => Some(e),
            Error::Gif(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

impl From<gif::EncodingError> for Error {
    fn from(e: gif::EncodingError) -> Self {
        Error::Gif(e)
    }
}
