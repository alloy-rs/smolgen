use std::{error::Error as StdError, fmt, ops::Deref};

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

    /// Wraps the error with a message, keeping it as the source.
    #[cfg(feature = "build")]
    pub(crate) fn context(self, message: &'static str) -> Self {
        Self(Box::new(Context {
            message,
            source: self.0,
        }))
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

impl Deref for Error {
    type Target = dyn StdError + Send + Sync;

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

impl fmt::Display for Error {
    /// Like `anyhow`, the alternate form (`{:#}`) appends the sources separated by `: `.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)?;
        if f.alternate() {
            let mut source = self.0.source();
            while let Some(error) = source {
                write!(f, ": {error}")?;
                source = error.source();
            }
        }
        Ok(())
    }
}

impl fmt::Debug for Error {
    /// Formats the error and its sources like `anyhow`'s `Debug`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)?;
        let Some(first) = self.0.source() else {
            return Ok(());
        };
        f.write_str("\n\nCaused by:")?;
        let multiple = first.source().is_some();
        let mut source = Some(first);
        let mut n = 0;
        while let Some(error) = source {
            let indent = if multiple {
                write!(f, "\n{n: >5}: ")?;
                "\n       "
            } else {
                f.write_str("\n    ")?;
                "\n    "
            };
            f.write_str(&error.to_string().replace('\n', indent))?;
            n += 1;
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

#[cfg(feature = "build")]
struct Context {
    message: &'static str,
    source: Box<dyn StdError + Send + Sync>,
}

#[cfg(feature = "build")]
impl fmt::Display for Context {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.message)
    }
}

#[cfg(feature = "build")]
impl fmt::Debug for Context {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.message)
    }
}

#[cfg(feature = "build")]
impl StdError for Context {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        Some(&*self.source)
    }
}
