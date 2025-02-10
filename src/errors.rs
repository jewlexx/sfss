pub trait RecoverableError {
    /// Checks if the error is recoverable
    fn recoverable(&self) -> bool;

    /// Returns `None` if the error is recoverable, or `Some(self)` if it is not
    fn into_recoverable(self) -> Option<Self>
    where
        Self: Sized,
    {
        if self.recoverable() { None } else { Some(self) }
    }
}

pub trait RecoverableResult<T> {
    /// Returns `None` if the error is recoverable, or `Some(self)` if it is not
    fn recoverable(self) -> Option<Self>
    where
        Self: Sized;
}

impl<T, E> RecoverableResult<T> for Result<T, E>
where
    E: RecoverableError,
{
    fn recoverable(self) -> Option<Self> {
        match self {
            Ok(v) => Some(Ok(v)),
            Err(e) => e.into_recoverable().map(Err),
        }
    }
}
