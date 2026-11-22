//! What the test binaries share.

use std::sync::{Mutex, MutexGuard, PoisonError};

/// The native effect and texture registries are not synchronized and the harness runs tests in
/// parallel: every test that makes GPU objects takes this first (declared before any GPU object, so
/// it is released after they drop).
static GPU: Mutex<()> = Mutex::new(());

pub fn gpu_lock() -> MutexGuard<'static, ()> {
    GPU.lock().unwrap_or_else(PoisonError::into_inner) // a failed test must not fail the rest
}
