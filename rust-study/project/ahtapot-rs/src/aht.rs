use core::fmt;
use image::ImageError;

#[derive(Debug)]
pub enum AhtError {
    Clap(clap::Error),
    Image(image::ImageError),
    Io(std::io::Error),
    Walk(walkdir::Error),
}

impl fmt::Display for AhtError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AhtError::Clap(c) => write!(f, "Clap Error: {}", c),
            AhtError::Image(i) => write!(f, "Image Error: {}", i),
            AhtError::Io(e) => write!(f, "IoError: {}", e),
            AhtError::Walk(w) => write!(f, "WalkDirError: {}", w),
        }
    }
}

impl From<std::io::Error> for AhtError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<clap::Error> for AhtError {
    fn from(value: clap::Error) -> Self {
        Self::Clap(value)
    }
}

impl From<ImageError> for AhtError {
    fn from(value: ImageError) -> Self {
        Self::Image(value)
    }
}

impl From<walkdir::Error> for AhtError {
    fn from(value: walkdir::Error) -> Self {
        Self::Walk(value)
    }
}
