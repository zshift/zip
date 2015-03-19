//! Error types that can be emitted from this library

use std::io;
use std::error;
use std::fmt;

/// Generic result type with ZipError as its error variant
pub type ZipResult<T> = Result<T, ZipError>;

/// Error type for Zip
#[derive(Debug)]
pub enum ZipError
{
    /// An Error caused by I/O
    Io(io::Error),

    /// This file is probably not a zip archive
    InvalidArchive(&'static str),

    /// This archive is not supported
    UnsupportedArchive(&'static str),

    /// The requested file could not be found in the archive
    FileNotFound,
}

impl ZipError
{
    fn detail(&self) -> ::std::borrow::Cow<str>
    {
        use ::std::error::Error;
        use ::std::borrow::IntoCow;

        match *self
        {
            ZipError::Io(ref io_err) => {
                ("Io Error: ".to_string() + (io_err as &error::Error).description()).into_cow()
            },
            ZipError::InvalidArchive(msg) | ZipError::UnsupportedArchive(msg) => {
                (self.description().to_string() + ": " + msg).into_cow()
            },
            ZipError::FileNotFound => {
                self.description().into_cow()
            },
        }
    }
}

impl error::FromError<io::Error> for ZipError
{
    fn from_error(err: io::Error) -> ZipError
    {
        ZipError::Io(err)
    }
}

impl fmt::Display for ZipError
{
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error>
    {
        fmt.write_str(&*self.detail())
    }
}

impl error::Error for ZipError
{
    fn description(&self) -> &str
    {
        match *self
        {
            ZipError::Io(ref io_err) => (io_err as &error::Error).description(),
            ZipError::InvalidArchive(..) => "Invalid Zip archive",
            ZipError::UnsupportedArchive(..) => "Unsupported Zip archive",
            ZipError::FileNotFound => "Specified file not found in archive",
        }
    }

    fn cause(&self) -> Option<&error::Error>
    {
        match *self
        {
            ZipError::Io(ref io_err) => Some(io_err as &error::Error),
            _ => None,
        }
    }
}
