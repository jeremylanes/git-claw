//! Advisory file locking on `slots.lock` with polling and timeout.

use crate::infra::error::LockError;
use fd_lock::{RwLock, RwLockWriteGuard};
use std::fs::{self, File, OpenOptions};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// RAII wrapper around an advisory write lock.
pub struct SlotLock {
    path: PathBuf,
    // Leaked reference to RwLock<File> allows the guard to be held in the struct.
    // Cleaned up on drop.
    guard: Option<RwLockWriteGuard<'static, File>>,
    raw_lock: *mut RwLock<File>,
}

impl SlotLock {
    pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);
    pub const POLL_INTERVAL: Duration = Duration::from_millis(50);

    /// Acquires an exclusive advisory file lock at `lock_path` with default 5-second timeout.
    pub fn acquire(lock_path: &Path) -> Result<Self, LockError> {
        Self::acquire_with_timeout(lock_path, Self::DEFAULT_TIMEOUT, Self::POLL_INTERVAL)
    }

    /// Acquires an exclusive advisory file lock with configurable timeout and poll interval.
    pub fn acquire_with_timeout(
        lock_path: &Path,
        timeout: Duration,
        poll_interval: Duration,
    ) -> Result<Self, LockError> {
        if let Some(parent) = lock_path.parent() {
            fs::create_dir_all(parent).map_err(|e| LockError::IoError {
                path: lock_path.to_string_lossy().to_string(),
                message: e.to_string(),
            })?;
        }

        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(lock_path)
            .map_err(|e| LockError::IoError {
                path: lock_path.to_string_lossy().to_string(),
                message: e.to_string(),
            })?;

        let raw_lock = Box::into_raw(Box::new(RwLock::new(file)));
        let start = Instant::now();

        loop {
            // SAFETY: raw_lock was allocated via Box::into_raw and remains valid until dropped.
            let rwlock_ref: &'static mut RwLock<File> = unsafe { &mut *raw_lock };

            match rwlock_ref.try_write() {
                Ok(guard) => {
                    return Ok(SlotLock {
                        path: lock_path.to_path_buf(),
                        guard: Some(guard),
                        raw_lock,
                    });
                }
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                    if start.elapsed() >= timeout {
                        unsafe {
                            let _ = Box::from_raw(raw_lock);
                        }
                        return Err(LockError::Timeout {
                            path: lock_path.to_string_lossy().to_string(),
                            timeout_secs: timeout.as_secs(),
                        });
                    }
                    std::thread::sleep(poll_interval);
                }
                Err(err) => {
                    unsafe {
                        let _ = Box::from_raw(raw_lock);
                    }
                    return Err(LockError::IoError {
                        path: lock_path.to_string_lossy().to_string(),
                        message: err.to_string(),
                    });
                }
            }
        }
    }

    /// Returns the path to the lock file.
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for SlotLock {
    fn drop(&mut self) {
        // Drop the guard first to release the OS advisory lock
        self.guard.take();
        // Deallocate the RwLock and close the underlying file
        if !self.raw_lock.is_null() {
            unsafe {
                let _ = Box::from_raw(self.raw_lock);
            }
            self.raw_lock = std::ptr::null_mut();
        }
    }
}

// SlotLock is Send across threads
unsafe impl Send for SlotLock {}
