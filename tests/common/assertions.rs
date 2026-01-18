//! Logged assertion helpers for rich_rust tests.
//!
//! These functions wrap standard assertions with tracing logs,
//! providing detailed context when assertions fail.

use std::fmt::Debug;

/// Assert equality with detailed logging.
///
/// Logs both values at debug level before comparison,
/// making it easier to diagnose failures in CI logs.
///
/// # Example
///
/// ```rust,ignore
/// assert_eq_logged("color count", colors.len(), 16);
/// ```
#[track_caller]
pub fn assert_eq_logged<T: PartialEq + Debug>(context: &str, actual: T, expected: T) {
    tracing::debug!(
        context = context,
        expected = ?expected,
        actual = ?actual,
        "asserting equality"
    );

    if actual != expected {
        tracing::error!(
            context = context,
            expected = ?expected,
            actual = ?actual,
            "assertion failed: values not equal"
        );
    }

    assert_eq!(
        actual, expected,
        "{context}: expected {expected:?}, got {actual:?}"
    );

    tracing::trace!(context = context, "assertion passed");
}

/// Assert that a value is true with logging.
///
/// # Example
///
/// ```rust,ignore
/// assert_true_logged("color is valid", color.is_valid());
/// ```
#[track_caller]
pub fn assert_true_logged(context: &str, value: bool) {
    tracing::debug!(context = context, value = value, "asserting true");

    if !value {
        tracing::error!(
            context = context,
            value = value,
            "assertion failed: expected true"
        );
    }

    assert!(value, "{context}: expected true, got false");

    tracing::trace!(context = context, "assertion passed");
}

/// Assert that a value is false with logging.
///
/// # Example
///
/// ```rust,ignore
/// assert_false_logged("should not be terminal", is_terminal);
/// ```
#[track_caller]
pub fn assert_false_logged(context: &str, value: bool) {
    tracing::debug!(context = context, value = value, "asserting false");

    if value {
        tracing::error!(
            context = context,
            value = value,
            "assertion failed: expected false"
        );
    }

    assert!(!value, "{context}: expected false, got true");

    tracing::trace!(context = context, "assertion passed");
}

/// Assert that a Result is Ok with logging.
///
/// Returns the Ok value for further assertions.
///
/// # Example
///
/// ```rust,ignore
/// let color = assert_ok_logged("parse color", Color::parse("#ff0000"));
/// ```
#[track_caller]
pub fn assert_ok_logged<T: Debug, E: Debug>(context: &str, result: Result<T, E>) -> T {
    tracing::debug!(context = context, result = ?result, "asserting Ok");

    match result {
        Ok(value) => {
            tracing::trace!(context = context, value = ?value, "assertion passed: got Ok");
            value
        }
        Err(ref e) => {
            tracing::error!(context = context, error = ?e, "assertion failed: expected Ok, got Err");
            panic!("{context}: expected Ok, got Err({e:?})");
        }
    }
}

/// Assert that a Result is Err with logging.
///
/// Returns the Err value for further assertions.
///
/// # Example
///
/// ```rust,ignore
/// let error = assert_err_logged("invalid color", Color::parse("not-a-color"));
/// ```
#[track_caller]
pub fn assert_err_logged<T: Debug, E: Debug>(context: &str, result: Result<T, E>) -> E {
    tracing::debug!(context = context, result = ?result, "asserting Err");

    match result {
        Err(e) => {
            tracing::trace!(context = context, error = ?e, "assertion passed: got Err");
            e
        }
        Ok(ref value) => {
            tracing::error!(
                context = context,
                value = ?value,
                "assertion failed: expected Err, got Ok"
            );
            panic!("{context}: expected Err, got Ok({value:?})");
        }
    }
}

/// Assert that an Option is Some with logging.
///
/// Returns the inner value for further assertions.
///
/// # Example
///
/// ```rust,ignore
/// let triplet = assert_some_logged("color triplet", color.triplet());
/// ```
#[track_caller]
pub fn assert_some_logged<T: Debug>(context: &str, option: Option<T>) -> T {
    tracing::debug!(context = context, option = ?option, "asserting Some");

    match option {
        Some(value) => {
            tracing::trace!(context = context, value = ?value, "assertion passed: got Some");
            value
        }
        None => {
            tracing::error!(
                context = context,
                "assertion failed: expected Some, got None"
            );
            panic!("{context}: expected Some, got None");
        }
    }
}

/// Assert that an Option is None with logging.
///
/// # Example
///
/// ```rust,ignore
/// assert_none_logged("default has no triplet", default_color.triplet());
/// ```
#[track_caller]
pub fn assert_none_logged<T: Debug>(context: &str, option: Option<T>) {
    tracing::debug!(context = context, option = ?option, "asserting None");

    if let Some(ref value) = option {
        tracing::error!(
            context = context,
            value = ?value,
            "assertion failed: expected None, got Some"
        );
        panic!("{context}: expected None, got Some({value:?})");
    }

    tracing::trace!(context = context, "assertion passed");
}

/// Assert that a string contains a substring with logging.
///
/// # Example
///
/// ```rust,ignore
/// assert_contains_logged("ansi output", &output, "\x1b[31m");
/// ```
#[track_caller]
pub fn assert_contains_logged(context: &str, haystack: &str, needle: &str) {
    tracing::debug!(
        context = context,
        haystack_len = haystack.len(),
        needle = needle,
        "asserting contains"
    );

    if !haystack.contains(needle) {
        tracing::error!(
            context = context,
            haystack = haystack,
            needle = needle,
            "assertion failed: string does not contain substring"
        );
        panic!(
            "{context}: expected string to contain {needle:?}, but it doesn't.\nString: {haystack:?}"
        );
    }

    tracing::trace!(context = context, "assertion passed");
}

/// Assert that a string does not contain a substring with logging.
///
/// # Example
///
/// ```rust,ignore
/// assert_not_contains_logged("plain output", &output, "\x1b[");
/// ```
#[track_caller]
pub fn assert_not_contains_logged(context: &str, haystack: &str, needle: &str) {
    tracing::debug!(
        context = context,
        haystack_len = haystack.len(),
        needle = needle,
        "asserting not contains"
    );

    if haystack.contains(needle) {
        tracing::error!(
            context = context,
            haystack = haystack,
            needle = needle,
            "assertion failed: string contains unwanted substring"
        );
        panic!(
            "{context}: expected string to not contain {needle:?}, but it does.\nString: {haystack:?}"
        );
    }

    tracing::trace!(context = context, "assertion passed");
}

/// Assert approximate equality for floating point values with logging.
///
/// Uses a relative epsilon for comparison.
///
/// # Example
///
/// ```rust,ignore
/// assert_approx_eq_logged("normalized red", normalized.0, 1.0, 0.001);
/// ```
#[track_caller]
pub fn assert_approx_eq_logged(context: &str, actual: f64, expected: f64, epsilon: f64) {
    tracing::debug!(
        context = context,
        expected = expected,
        actual = actual,
        epsilon = epsilon,
        "asserting approximate equality"
    );

    let diff = (actual - expected).abs();
    if diff > epsilon {
        tracing::error!(
            context = context,
            expected = expected,
            actual = actual,
            diff = diff,
            epsilon = epsilon,
            "assertion failed: values not approximately equal"
        );
        panic!("{context}: expected {expected} (within {epsilon}), got {actual} (diff: {diff})");
    }

    tracing::trace!(context = context, "assertion passed");
}

/// Assert that a slice has a specific length with logging.
///
/// # Example
///
/// ```rust,ignore
/// assert_len_logged("segments", &segments, 5);
/// ```
#[track_caller]
pub fn assert_len_logged<T>(context: &str, slice: &[T], expected_len: usize) {
    let actual_len = slice.len();
    tracing::debug!(
        context = context,
        expected_len = expected_len,
        actual_len = actual_len,
        "asserting length"
    );

    if actual_len != expected_len {
        tracing::error!(
            context = context,
            expected_len = expected_len,
            actual_len = actual_len,
            "assertion failed: unexpected length"
        );
        panic!("{context}: expected length {expected_len}, got {actual_len}");
    }

    tracing::trace!(context = context, "assertion passed");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::init_test_logging;

    #[test]
    fn test_assert_eq_logged_pass() {
        init_test_logging();
        assert_eq_logged("simple equality", 42, 42);
    }

    #[test]
    #[should_panic(expected = "expected 42")]
    fn test_assert_eq_logged_fail() {
        init_test_logging();
        assert_eq_logged("will fail", 0, 42);
    }

    #[test]
    fn test_assert_ok_logged_pass() {
        init_test_logging();
        let result: Result<i32, &str> = Ok(42);
        let value = assert_ok_logged("ok result", result);
        assert_eq!(value, 42);
    }

    #[test]
    fn test_assert_contains_logged_pass() {
        init_test_logging();
        assert_contains_logged("substring", "hello world", "world");
    }

    #[test]
    fn test_assert_len_logged_pass() {
        init_test_logging();
        let vec = vec![1, 2, 3, 4, 5];
        assert_len_logged("vec length", &vec, 5);
    }
}
