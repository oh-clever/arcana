macro_rules! internal_err {
    ($message:expr) => {
        Err(InternalError::new($message))
    };
    ($message:expr, $($fmt_args:expr),*) => {
        Err(InternalError::new(format!($message, $($fmt_args),*)))
    };
}

pub(crate) use internal_err;
