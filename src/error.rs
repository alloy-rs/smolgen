use std::{error::Error as StdError, fmt};

/// The error type used by `smolgen`.
///
/// Like `anyhow::Error`, it converts from any [`std::error::Error`] with `?`, and converts
/// into `Box<dyn std::error::Error>` so build scripts returning that type can use `?` on it.
///
/// ```
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     smolgen::Emitter::default().emit()?;
///     Ok(())
/// }
/// ```
pub struct Error(Box<dyn StdError + Send + Sync>);

impl Error {
    /// Creates an error from a message.
    pub fn msg<M: fmt::Display + fmt::Debug + Send + Sync + 'static>(message: M) -> Self {
        Self(Box::new(Message(message)))
    }
}

impl<E: StdError + Send + Sync + 'static> From<E> for Error {
    fn from(error: E) -> Self {
        Self(Box::new(error))
    }
}

impl From<Error> for Box<dyn StdError + Send + Sync> {
    fn from(error: Error) -> Self {
        error.0
    }
}

impl From<Error> for Box<dyn StdError + Send> {
    fn from(error: Error) -> Self {
        error.0
    }
}

impl From<Error> for Box<dyn StdError> {
    fn from(error: Error) -> Self {
        error.0
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)?;
        let mut source = self.0.source();
        if source.is_some() {
            f.write_str("\n\nCaused by:")?;
        }
        while let Some(error) = source {
            write!(f, "\n    {error}")?;
            source = error.source();
        }
        Ok(())
    }
}

struct Message<M>(M);

impl<M: fmt::Display> fmt::Display for Message<M> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<M: fmt::Debug> fmt::Debug for Message<M> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<M: fmt::Display + fmt::Debug> StdError for Message<M> {}
