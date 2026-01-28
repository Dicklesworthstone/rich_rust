//! # Mutex Poison Handling Strategy — Design RFC
//!
//! ## Context
//!
//! `rich_rust` uses `std::sync::Mutex` and `std::sync::RwLock` in several places:
//!
//! | Module | Lock type | Protects | Current pattern |
//! |--------|-----------|----------|-----------------|
//! | `cells.rs` | `Mutex<LruCache>` | Cell width cache | `if let Ok(..)` (silent skip) |
//! | `style.rs` | `Mutex<LruCache>` (x2) | ANSI/parse caches | `if let Ok(..)` (silent skip) |
//! | `logging.rs` | `Mutex<Option<String>>` | Last timestamp | `unwrap_or_else(into_inner)` |
//! | `live.rs` | `Mutex` (x4) + `RwLock` (x1) | Live display state | Mixed patterns |
//! | `state.rs` (demo) | `Mutex<DemoState>` | Demo app state | `unwrap_or_else(into_inner)` |
//!
//! Three inconsistent patterns exist today:
//!
//! 1. **Silent skip** — `if let Ok(guard) = mutex.lock()` — cache misses silently on poison
//! 2. **Recover** — `.unwrap_or_else(PoisonError::into_inner)` — extracts data, continues
//! 3. **Panic** — `.unwrap()` — only in test code
//!
//! ## Decision
//!
//! **Chosen strategy: Always recover, with debug-mode logging.**
//!
//! Rationale:
//!
//! - `rich_rust` is an **output library**, not a database or transaction system. Mutex-protected
//!   data is either a cache (where staleness is harmless) or display state (where a best-effort
//!   render is better than a panic).
//! - A thread panicking while holding a lock should **not** bring down the entire application.
//!   The library consumer's program should keep running.
//! - In debug builds, we log poison recovery events to `eprintln!` so developers can track
//!   down the originating panic. In release builds, recovery is silent and zero-overhead.
//!
//! ## Helper Categories
//!
//! | Helper | Use when |
//! |--------|----------|
//! | [`lock_recover`] | Default for all `Mutex` acquisitions |
//! | [`lock_recover_debug`] | When you want debug-mode logging with context |
//! | [`read_recover`] | Default for all `RwLock` read acquisitions |
//! | [`write_recover`] | Default for all `RwLock` write acquisitions |
//!
//! ## Migration Plan
//!
//! All existing lock sites should be migrated to use these helpers:
//!
//! - **Caches** (`cells.rs`, `style.rs`): Replace `if let Ok(..)` with `lock_recover`.
//!   A poisoned cache still contains valid (if possibly stale) data; recovering is strictly
//!   better than silently skipping the cache lookup.
//! - **State** (`logging.rs`, `live.rs`, `state.rs`): Replace ad-hoc `unwrap_or_else` with
//!   `lock_recover` or `lock_recover_debug` for consistency.
//! - **Tests**: May continue using `.unwrap()` since test panics are expected.
//!
//! ## Examples
//!
//! ```rust,ignore
//! use std::sync::Mutex;
//! use rich_rust::sync::lock_recover;
//!
//! let mutex = Mutex::new(42);
//! let guard = lock_recover(&mutex);
//! assert_eq!(*guard, 42);
//! ```
//!
//! ```rust,ignore
//! use std::sync::Mutex;
//! use rich_rust::sync::lock_recover_debug;
//!
//! let mutex = Mutex::new(vec![1, 2, 3]);
//! let guard = lock_recover_debug(&mutex, "style_cache_lookup");
//! assert_eq!(guard.len(), 3);
//! ```

use std::sync::{Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};

/// Lock a `Mutex`, recovering from poison if necessary.
///
/// If the mutex is healthy, returns the guard normally. If the mutex is poisoned
/// (a thread panicked while holding the lock), extracts the inner data and returns
/// the guard anyway.
///
/// This is the correct default for `rich_rust` because all mutex-protected data is
/// either a cache (where staleness is harmless) or display state (where a best-effort
/// render beats crashing the consumer's application).
#[inline]
pub fn lock_recover<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// Lock a `Mutex` with debug-mode poison logging.
///
/// Identical to [`lock_recover`] but emits a diagnostic message via `eprintln!` in
/// debug builds when recovering from poison. Release builds compile to the same code
/// as `lock_recover` (the `context` parameter is unused and optimized away).
///
/// Use this variant when you want observability during development — for example,
/// in high-traffic lock sites where a poisoned mutex might indicate a deeper bug.
#[inline]
#[allow(clippy::used_underscore_binding)]
pub fn lock_recover_debug<'a, T>(mutex: &'a Mutex<T>, _context: &str) -> MutexGuard<'a, T> {
    mutex.lock().unwrap_or_else(|e| {
        #[cfg(debug_assertions)]
        eprintln!("[mutex-poison] recovered at: {_context}");
        e.into_inner()
    })
}

/// Acquire a read lock on an `RwLock`, recovering from poison if necessary.
///
/// Semantics mirror [`lock_recover`] for the read side of an `RwLock`.
#[inline]
pub fn read_recover<T>(rwlock: &RwLock<T>) -> RwLockReadGuard<'_, T> {
    rwlock
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// Acquire a write lock on an `RwLock`, recovering from poison if necessary.
///
/// Semantics mirror [`lock_recover`] for the write side of an `RwLock`.
#[inline]
pub fn write_recover<T>(rwlock: &RwLock<T>) -> RwLockWriteGuard<'_, T> {
    rwlock
        .write()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::panic;

    #[test]
    fn lock_recover_normal() {
        let mutex = Mutex::new(42);
        let guard = lock_recover(&mutex);
        assert_eq!(*guard, 42);
    }

    #[test]
    fn lock_recover_poisoned() {
        let mutex = Mutex::new(42);
        // Poison the mutex by panicking while holding the lock.
        let _ = panic::catch_unwind(panic::AssertUnwindSafe(|| {
            let _guard = mutex.lock().unwrap();
            panic!("intentional panic to poison mutex");
        }));
        assert!(mutex.is_poisoned());
        let guard = lock_recover(&mutex);
        assert_eq!(*guard, 42);
    }

    #[test]
    fn lock_recover_debug_normal() {
        let mutex = Mutex::new("hello");
        let guard = lock_recover_debug(&mutex, "test_context");
        assert_eq!(*guard, "hello");
    }

    #[test]
    fn lock_recover_debug_poisoned() {
        let mutex = Mutex::new(vec![1, 2, 3]);
        let _ = panic::catch_unwind(panic::AssertUnwindSafe(|| {
            let _guard = mutex.lock().unwrap();
            panic!("intentional panic to poison mutex");
        }));
        assert!(mutex.is_poisoned());
        let guard = lock_recover_debug(&mutex, "test_poisoned_vec");
        assert_eq!(*guard, vec![1, 2, 3]);
    }

    #[test]
    fn read_recover_normal() {
        let rwlock = RwLock::new(42);
        let guard = read_recover(&rwlock);
        assert_eq!(*guard, 42);
    }

    #[test]
    fn write_recover_normal() {
        let rwlock = RwLock::new(42);
        let mut guard = write_recover(&rwlock);
        *guard = 100;
        drop(guard);
        assert_eq!(*read_recover(&rwlock), 100);
    }

    #[test]
    fn write_recover_poisoned() {
        let rwlock = RwLock::new(42);
        let _ = panic::catch_unwind(panic::AssertUnwindSafe(|| {
            let _guard = rwlock.write().unwrap();
            panic!("intentional panic to poison rwlock");
        }));
        assert!(rwlock.is_poisoned());
        let mut guard = write_recover(&rwlock);
        *guard = 99;
        drop(guard);
        assert_eq!(*read_recover(&rwlock), 99);
    }

    #[test]
    fn read_recover_after_write_poison() {
        let rwlock = RwLock::new(42);
        let _ = panic::catch_unwind(panic::AssertUnwindSafe(|| {
            let _guard = rwlock.write().unwrap();
            panic!("intentional panic to poison rwlock");
        }));
        assert!(rwlock.is_poisoned());
        let guard = read_recover(&rwlock);
        assert_eq!(*guard, 42);
    }
}
