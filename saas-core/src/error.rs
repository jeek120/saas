use crate::BoxError;
use std::{error::Error as StdError, fmt};

// Saas错误
#[derive(Debug)]
pub struct Error {
    inner: BoxError,
}

impl Error {
    /// 创建一个新的错误从boxerror
    pub fn new(error: impl Into<BoxError>) -> Self {
        Self {
            inner: error.into()
        }
    }

    /// 还源一个 `Error` 到一个Box特性的的对象
    pub fn into_inner(self) -> BoxError {
        self.inner
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        Some(&*self.inner)
    }
}