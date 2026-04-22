use std::*;

pub struct Error {
    err: String,
}
impl error::Error for Error {}
impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.err)
    }
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.err)
    }
}

pub struct ErrorLocation<E> {
    err: Box<E>,
    loc: &'static panic::Location<'static>,
}
impl<E: fmt::Debug> ErrorLocation<E> {
    fn _get(self) -> E {
        *self.err
    }
    fn track(&self) -> String {
        format!(
            "{}:{}:{}",
            self.loc.file(),
            self.loc.line(),
            self.loc.column()
        )
    }
}
unsafe impl<E> Send for ErrorLocation<E> {}
impl<E: fmt::Debug + fmt::Display> error::Error for ErrorLocation<E> {}
impl<E: fmt::Debug> fmt::Display for ErrorLocation<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:[{:?}]", self.track(), self.err)
    }
}
impl<E: fmt::Debug> fmt::Debug for ErrorLocation<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            write!(f, "{}:\n[\n\t{:#?}\n]", self.track(), self.err)
        } else {
            write!(f, "{}:[{:?}]", self.track(), self.err)
        }
    }
}
// for std errror
pub trait ErrorCaller<T, E> {
    fn e(self) -> Result<T, ErrorLocation<E>>;
}
impl<T, E: fmt::Debug + fmt::Display> ErrorCaller<T, E> for Result<T, E> {
    #[track_caller]
    fn e(self) -> Result<T, ErrorLocation<E>> {
        if self.is_ok() {
            let r = unsafe { self.unwrap_unchecked() };
            return Ok(r);
        } else {
            let e = unsafe { self.unwrap_err_unchecked() };
            let loc = panic::Location::caller();
            return Err(ErrorLocation {
                err: Box::new(e),
                loc,
            });
        }
    }
}

pub trait Caller<T> {
    fn o(self) -> Result<T, ErrorLocation<Error>>;
}
// for option
impl<T> Caller<T> for Option<T> {
    #[track_caller]
    fn o(self) -> Result<T, ErrorLocation<Error>> {
        if self.is_some() {
            let o = unsafe { self.unwrap_unchecked() };
            return Ok(o);
        } else {
            let e = Error {
                err: format!("is None"),
            };
            let loc = panic::Location::caller();
            return Err(ErrorLocation {
                err: Box::new(e),
                loc,
            });
        }
    }
}
// for thread handle
impl<T> Caller<T> for Result<T, Box<dyn any::Any + Send>> {
    #[track_caller]
    fn o(self) -> Result<T, ErrorLocation<Error>> {
        if self.is_ok() {
            let r = unsafe { self.unwrap_unchecked() };
            return Ok(r);
        } else {
            let e = unsafe { self.unwrap_err_unchecked() };
            let err = if let Some(e) = e.downcast_ref::<&str>() {
                e.to_string()
            } else if let Ok(e) = e.downcast::<String>() {
                *e
            } else {
                "unknown handle panic".into()
            };
            let loc = panic::Location::caller();
            return Err(ErrorLocation {
                err: Box::new(Error { err }),
                loc,
            });
        }
    }
}
// for mutex poison
impl<T> Caller<T> for Result<T, sync::PoisonError<T>> {
    #[track_caller]
    fn o(self) -> Result<T, ErrorLocation<Error>> {
        if self.is_ok() {
            let r = unsafe { self.unwrap_unchecked() };
            return Ok(r);
        } else {
            let e = unsafe { self.unwrap_err_unchecked() };
            let err = e.to_string();
            let loc = panic::Location::caller();
            return Err(ErrorLocation {
                err: Box::new(Error { err }),
                loc,
            });
        }
    }
}
#[track_caller]
/// Creates new error from string
pub fn new(err: String) -> ErrorLocation<Error> {
    let loc = panic::Location::caller();
    ErrorLocation {
        err: Box::new(Error { err }),
        loc,
    }
}
