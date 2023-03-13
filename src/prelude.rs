use std::{backtrace::Backtrace, fmt::Display};

use log::error;

trait DisplayBackTrace {
    fn to_simple_string(&self) -> String;
}

impl DisplayBackTrace for Backtrace {
    fn to_simple_string(&self) -> String {
        let full = format!("{}", self);
        let split = full.split('\n');
        let mut trimmed = String::new();
        for (i, line) in split.enumerate() {
            if (8..32).contains(&i) {
                trimmed.push_str(line);
                trimmed.push('\n');
            }
        }
        trimmed
    }
}

pub trait EnhancedUnwrap<T> {
    /// Equivalent to [`Option::unwrap`] & [`Result::unwrap`] with additional logging
    fn unwp(self) -> T;
}

pub trait EnhancedExpect<T, E: Display> {
    /// Equivalent to [`Option::expect`] & [`Result::expect`] with additional logging.
    /// [`EnhancedExpect::ex`] stands for Expect, Extra(logging), Exception, Enhanced
    fn ex(self, msg: &str) -> T;
}

impl<T, E: Display> EnhancedUnwrap<T> for Result<T, E> {
    #[inline]
    fn unwp(self) -> T {
        ok(self)
    }
}

impl<T, E: Display> EnhancedExpect<T, E> for Result<T, E> {
    #[inline]
    fn ex(self, msg: &str) -> T {
        ok_ctx(self, msg)
    }
}

impl<T> EnhancedUnwrap<T> for Option<T> {
    #[inline]
    fn unwp(self) -> T {
        some(self)
    }
}

impl<T> EnhancedExpect<T, String> for Option<T> {
    #[inline]
    fn ex(self, msg: &str) -> T {
        some_ctx(self, msg)
    }
}

#[inline]
pub fn ok<T, E: Display>(result: Result<T, E>) -> T {
    ok_ctx(result, "")
}

#[inline]
pub fn some<T>(option: Option<T>) -> T {
    some_ctx(option, "")
}

/// [`Result`] should be ok with custom context
#[inline]
pub fn ok_ctx<T, E: Display>(result: Result<T, E>, msg: &str) -> T {
    match result {
        Ok(value) => value,
        Err(e) => {
            log_and_panic(Some(e), msg);
        }
    }
}

/// [`Option`] should be some with custom context
#[inline]
pub fn some_ctx<T>(option: Option<T>, msg: &str) -> T {
    match option {
        Some(value) => value,
        None => {
            log_and_panic::<String>(None, msg);
        }
    }
}

#[inline]
fn log_and_panic<E: Display>(err: Option<E>, msg: &str) -> ! {
    let err_msg = match err {
        Some(e) => format!("{}", e),
        None => "".to_string(),
    };

    let info = format!(
        "this should never happen: {}, context: {}, back_trace: {}",
        err_msg,
        msg,
        Backtrace::force_capture().to_simple_string()
    );
    error!("{}", info);
    panic!("{}", info);
}
