use std::{
    fs::{File, OpenOptions, create_dir, create_dir_all, remove_dir, remove_dir_all, remove_file},
    io,
    ops::{Deref, DerefMut},
    path::Path,
};

/// RAII guard, which calls [`(*self).unlock()`](std::fs::File::unlock) on drop.
#[derive(Debug)]
pub struct FileLockGuard {
    file: File,
}

impl Drop for FileLockGuard {
    fn drop(&mut self) {}
}

impl Deref for FileLockGuard {
    type Target = File;

    fn deref(&self) -> &Self::Target {
        &self.file
    }
}

impl DerefMut for FileLockGuard {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.file
    }
}

/// Options for controlling [`PathExt::mkdir`](PathExt::mkdir).
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub enum MkdirOptions {
    /// Equivalent of `mkdir $path`.
    WithoutParents,
    /// Equivalent of `mkdir -p $path`.
    WithParents,
}

mod sealed {
    use std::path::Path;

    pub trait Sealed {}
    impl Sealed for Path {}
}

/// Whether [`PathExt::lock`](PathExt::lock)/[`PathExt::lock_shared`](PathExt::lock_shared) should block current thread.
#[derive(Debug, Hash, Clone, Copy, PartialEq, Eq)]
pub enum ShouldBlock {
    No,
    Yes,
}

pub trait PathExt: sealed::Sealed {
    /// Touch file and its parent directories.
    ///
    /// # Returns
    /// [`Ok(File)`](std::fs::File) if created successfully, otherwise error, as reported by
    /// [`PathExt::mkdir`](PathExt::mkdir), or
    /// [`OpenOptions::open`](std::fs::OpenOptions::open).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::path::PathBuf;
    /// # use rustvil::fs::path_ext::PathExt;
    /// # use std::error::Error;
    /// # use std::fs::remove_file;
    ///
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// let buf = PathBuf::from("file.txt");
    /// let path = buf.as_path();
    /// let file = path.touch()?;
    /// # let _ = path.rm();
    /// # Ok(())
    /// # }
    /// ```
    fn touch(&self) -> io::Result<File>;

    /// Create directories at given [`Path`](std::path::Path).
    ///
    /// # Returns
    /// `Ok(())` if created successfully, otherwise error, as reported by
    /// [`create_dir`](std::fs::create_dir), or
    /// [`create_dir_all`](std::fs::create_dir_all).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::path::PathBuf;
    /// # use rustvil::fs::path_ext::{PathExt, MkdirOptions};
    /// # use std::error::Error;
    ///
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// let buf = PathBuf::from("a/b");
    /// let path = buf.as_path();
    /// path.mkdir(MkdirOptions::WithParents)?;
    /// # let _ = PathBuf::from("a").rmtree();
    /// # Ok(())
    /// # }
    /// ```
    fn mkdir(&self, opts: MkdirOptions) -> io::Result<()>;

    /// Wrapper around [`std::fs::remove_dir`](std::fs::remove_dir).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::path::PathBuf;
    /// # use rustvil::fs::path_ext::{PathExt, MkdirOptions};
    /// # use std::error::Error;
    ///
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// let buf = PathBuf::from("empty_dir");
    /// let path = buf.as_path();
    /// path.mkdir(MkdirOptions::WithoutParents)?;
    /// path.rmdir()?;
    /// # Ok(())
    /// # }
    /// ```
    fn rmdir(&self) -> io::Result<()>;

    /// Wrapper around [`std::fs::remove_dir_all`](std::fs::remove_dir_all).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::path::PathBuf;
    /// # use rustvil::fs::path_ext::{PathExt, MkdirOptions};
    /// # use std::error::Error;
    ///
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// let buf = PathBuf::from("a/b/c");
    /// let path = buf.as_path();
    /// path.mkdir(MkdirOptions::WithParents)?;
    /// let root = PathBuf::from("a");
    /// root.rmtree()?;
    /// # Ok(())
    /// # }
    /// ```
    fn rmtree(&self) -> io::Result<()>;

    /// Wrapper around [`std::fs::remove_file`](std::fs::remove_file).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::path::PathBuf;
    /// # use rustvil::fs::path_ext::PathExt;
    /// # use std::error::Error;
    ///
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// let buf = PathBuf::from("file.txt");
    /// let path = buf.as_path();
    /// path.touch()?;
    /// path.rm()?;
    /// # Ok(())
    /// # }
    /// ```
    fn rm(&self) -> io::Result<()>;

    /// Locks exclusively `self`, creating file if needed.
    ///
    /// # Returns
    /// [`Ok(FileLockGuard)`](FileLockGuard) on success.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::path::PathBuf;
    /// # use rustvil::fs::path_ext::{PathExt, ShouldBlock};
    /// # use std::error::Error;
    ///
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// let buf = PathBuf::from("lockfile.lock");
    /// let path = buf.as_path();
    ///
    /// // Acquire exclusive lock (blocking)
    /// let lock = path.lock(ShouldBlock::Yes)?;
    /// // Critical section...
    /// drop(lock); // Release lock
    ///
    /// // Try to acquire lock without blocking
    /// let lock = path.lock(ShouldBlock::No)?;
    /// drop(lock);
    /// # let _ = path.rm();
    /// # Ok(())
    /// # }
    /// ```
    fn lock(&self, should_block: ShouldBlock) -> io::Result<FileLockGuard>;

    /// Locks shared `self`, creating file if needed.
    ///
    /// # Returns
    /// [`Ok(FileLockGuard)`](FileLockGuard) on success.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::path::PathBuf;
    /// # use rustvil::fs::path_ext::{PathExt, ShouldBlock};
    /// # use std::error::Error;
    ///
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// let buf = PathBuf::from("lockfile.lock");
    /// let path = buf.as_path();
    ///
    /// // Acquire shared lock (blocking)
    /// let lock1 = path.lock_shared(ShouldBlock::Yes)?;
    /// // Multiple shared locks can coexist
    /// let lock2 = path.lock_shared(ShouldBlock::No)?;
    /// drop(lock1);
    /// drop(lock2);
    /// # let _ = path.rm();
    /// # Ok(())
    /// # }
    /// ```
    fn lock_shared(&self, should_block: ShouldBlock) -> io::Result<FileLockGuard>;
}

impl PathExt for Path {
    // FIXME: Take `OpenOptions` as a parameter?
    fn touch(&self) -> io::Result<File> {
        if let Some(parent) = self.parent() {
            {
                let opts = MkdirOptions::WithParents;
                match opts {
                    MkdirOptions::WithoutParents => create_dir(parent),
                    MkdirOptions::WithParents => create_dir_all(parent),
                }
            }?;
        }
        let mut opts = OpenOptions::new();
        opts.read(true).write(true).create(true).truncate(false);
        #[cfg(unix)]
        {
            use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
            if let Ok(metadata) = self.metadata() {
                // RDWR for all.
                const MASK: u32 = 0o666;
                let permissions = metadata.permissions().mode();
                opts.mode(permissions & MASK);
            }
        }
        opts.open(self)
    }

    fn mkdir(&self, opts: MkdirOptions) -> io::Result<()> {
        match opts {
            MkdirOptions::WithoutParents => create_dir(self),
            MkdirOptions::WithParents => create_dir_all(self),
        }
    }

    fn rmdir(&self) -> io::Result<()> {
        remove_dir(self)
    }

    fn rmtree(&self) -> io::Result<()> {
        remove_dir_all(self)
    }

    fn rm(&self) -> io::Result<()> {
        remove_file(self)
    }

    fn lock(&self, should_block: ShouldBlock) -> io::Result<FileLockGuard> {
        let file = self.touch()?;
        let result = if matches!(should_block, ShouldBlock::Yes) {
            file.lock()
        } else {
            file.try_lock().map_err(|err| match err {
                std::fs::TryLockError::Error(error) => error,
                std::fs::TryLockError::WouldBlock => io::Error::from(io::ErrorKind::WouldBlock),
            })
        };
        result.map(|_| FileLockGuard { file })
    }

    fn lock_shared(&self, should_block: ShouldBlock) -> io::Result<FileLockGuard> {
        let file = self.touch()?;
        let result = if matches!(should_block, ShouldBlock::Yes) {
            file.lock_shared()
        } else {
            file.try_lock_shared().map_err(|err| match err {
                std::fs::TryLockError::Error(error) => error,
                std::fs::TryLockError::WouldBlock => io::Error::from(io::ErrorKind::WouldBlock),
            })
        };
        result.map(|_| FileLockGuard { file })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use claim::{assert_err, assert_ok};
    use tempfile::tempdir;

    use std::sync::{Arc, Barrier};
    use std::thread;
    use std::time::Duration;
    use tempfile::NamedTempFile;

    #[test]
    fn create_new_file_should_work() {
        let tmp = tempdir().expect("needed for tests");
        let mut new_file = tmp.path().to_path_buf();
        new_file.push("x");
        assert_ok!(new_file.touch());
    }

    #[test]
    fn create_dirs() {
        {
            let tmp = tempdir().expect("needed for tests");
            let mut new_file = tmp.path().to_path_buf();
            new_file.push("x");
            new_file.push("y");
            assert_ok!(new_file.mkdir(MkdirOptions::WithParents));
        }

        {
            let tmp = tempdir().expect("needed for tests");
            let mut new_file = tmp.path().to_path_buf();
            new_file.push("x");
            assert_ok!(new_file.mkdir(MkdirOptions::WithParents));
        }

        {
            let tmp = tempdir().expect("needed for tests");
            let mut new_file = tmp.path().to_path_buf();
            new_file.push("x");
            assert_ok!(new_file.mkdir(MkdirOptions::WithoutParents));
        }

        {
            let tmp = tempdir().expect("needed for tests");
            let mut new_file = tmp.path().to_path_buf();
            new_file.push("x");
            new_file.push("y");
            assert_err!(new_file.mkdir(MkdirOptions::WithoutParents));
        }
    }

    #[test]
    fn lock_blocking_should_work() {
        use tempfile::NamedTempFile;
        let lockfile = NamedTempFile::new().expect("needed for tests");
        let path = lockfile.path();

        let lock = assert_ok!(path.lock(ShouldBlock::Yes));
        drop(lock);
    }

    #[test]
    fn lock_non_blocking_should_work() {
        use tempfile::NamedTempFile;
        let lockfile = NamedTempFile::new().expect("needed for tests");
        let path = lockfile.path();

        let lock = assert_ok!(path.lock(ShouldBlock::No));
        drop(lock);
    }

    #[test]
    fn lock_shared_blocking_should_work() {
        use tempfile::NamedTempFile;
        let lockfile = NamedTempFile::new().expect("needed for tests");
        let path = lockfile.path();

        let lock = assert_ok!(path.lock_shared(ShouldBlock::Yes));
        drop(lock);
    }

    #[test]
    fn lock_shared_non_blocking_should_work() {
        use tempfile::NamedTempFile;
        let lockfile = NamedTempFile::new().expect("needed for tests");
        let path = lockfile.path();

        let lock = assert_ok!(path.lock_shared(ShouldBlock::No));
        drop(lock);
    }

    #[test]
    fn multiple_shared_locks_can_coexist() {
        use tempfile::NamedTempFile;
        let lockfile = NamedTempFile::new().expect("needed for tests");
        let path = lockfile.path();

        let lock1 = assert_ok!(path.lock_shared(ShouldBlock::Yes));
        let lock2 = assert_ok!(path.lock_shared(ShouldBlock::No));
        let lock3 = assert_ok!(path.lock_shared(ShouldBlock::No));

        drop(lock1);
        drop(lock2);
        drop(lock3);
    }

    #[test]
    fn exclusive_lock_prevents_another_exclusive_lock() {
        use tempfile::NamedTempFile;
        let lockfile = NamedTempFile::new().expect("needed for tests");
        let path = lockfile.path();

        let _lock1 = assert_ok!(path.lock(ShouldBlock::Yes));
        let result = path.lock(ShouldBlock::No);

        assert_err!(&result);
        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::WouldBlock);
    }

    #[test]
    fn exclusive_lock_prevents_shared_lock() {
        use tempfile::NamedTempFile;
        let lockfile = NamedTempFile::new().expect("needed for tests");
        let path = lockfile.path();

        let _lock1 = assert_ok!(path.lock(ShouldBlock::Yes));
        let result = path.lock_shared(ShouldBlock::No);

        assert_err!(&result);
        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::WouldBlock);
    }

    #[test]
    fn shared_lock_prevents_exclusive_lock() {
        use tempfile::NamedTempFile;
        let lockfile = NamedTempFile::new().expect("needed for tests");
        let path = lockfile.path();

        let _lock1 = assert_ok!(path.lock_shared(ShouldBlock::Yes));
        let result = path.lock(ShouldBlock::No);

        assert_err!(&result);
        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::WouldBlock);
    }

    #[test]
    fn multithreaded_exclusive_locks_are_mutually_exclusive() {
        let lockfile = NamedTempFile::new().expect("needed for tests");
        let lockpath = Arc::new(lockfile.path().to_path_buf());
        let barrier = Arc::new(Barrier::new(2));

        let lockpath_clone = Arc::clone(&lockpath);
        let barrier_clone = Arc::clone(&barrier);

        let handle = thread::spawn(move || {
            let lock = assert_ok!(lockpath_clone.as_path().lock(ShouldBlock::Yes));
            barrier_clone.wait();
            thread::sleep(Duration::from_millis(100));
            drop(lock);
        });

        barrier.wait();
        thread::sleep(Duration::from_millis(10));

        let result = lockpath.as_path().lock(ShouldBlock::No);
        assert_err!(&result);
        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::WouldBlock);

        handle.join().expect("thread should not panic");

        assert_ok!(lockpath.as_path().lock(ShouldBlock::No));
    }

    #[test]
    fn multithreaded_shared_locks_can_coexist() {
        let lockfile = NamedTempFile::new().expect("needed for tests");
        let lockpath = Arc::new(lockfile.path().to_path_buf());
        let barrier = Arc::new(Barrier::new(3));

        let mut handles = vec![];

        for _ in 0..2 {
            let lockpath_clone = Arc::clone(&lockpath);
            let barrier_clone = Arc::clone(&barrier);

            let handle = thread::spawn(move || {
                let _lock = assert_ok!(lockpath_clone.as_path().lock_shared(ShouldBlock::Yes));
                barrier_clone.wait();
            });

            handles.push(handle);
        }

        let _lock = assert_ok!(lockpath.as_path().lock_shared(ShouldBlock::Yes));
        barrier.wait();

        for handle in handles {
            handle.join().expect("thread should not panic");
        }
    }

    #[test]
    fn multithreaded_exclusive_lock_blocks_shared_locks() {
        let lockfile = NamedTempFile::new().expect("needed for tests");
        let lockpath = Arc::new(lockfile.path().to_path_buf());
        let barrier = Arc::new(Barrier::new(2));

        let lockpath_clone = Arc::clone(&lockpath);
        let barrier_clone = Arc::clone(&barrier);

        let handle = thread::spawn(move || {
            let lock = assert_ok!(lockpath_clone.as_path().lock(ShouldBlock::Yes));
            barrier_clone.wait();
            thread::sleep(Duration::from_millis(100));
            drop(lock);
        });

        barrier.wait();
        thread::sleep(Duration::from_millis(10));

        let result = lockpath.as_path().lock_shared(ShouldBlock::No);
        assert_err!(&result);
        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::WouldBlock);

        handle.join().expect("thread should not panic");

        assert_ok!(lockpath.as_path().lock_shared(ShouldBlock::No));
    }

    #[test]
    fn multithreaded_shared_lock_blocks_exclusive_lock() {
        let lockfile = NamedTempFile::new().expect("needed for tests");
        let lockpath = Arc::new(lockfile.path().to_path_buf());
        let barrier = Arc::new(Barrier::new(2));

        let lockpath_clone = Arc::clone(&lockpath);
        let barrier_clone = Arc::clone(&barrier);

        let handle = thread::spawn(move || {
            let lock = assert_ok!(lockpath_clone.as_path().lock_shared(ShouldBlock::Yes));
            barrier_clone.wait();
            thread::sleep(Duration::from_millis(100));
            drop(lock);
        });

        barrier.wait();
        thread::sleep(Duration::from_millis(10));

        let result = lockpath.as_path().lock(ShouldBlock::No);
        assert_err!(&result);
        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::WouldBlock);

        handle.join().expect("thread should not panic");

        assert_ok!(lockpath.as_path().lock(ShouldBlock::No));
    }

    #[test]
    fn multithreaded_multiple_shared_then_exclusive() {
        let lockfile = NamedTempFile::new().expect("needed for tests");
        let lockpath = Arc::new(lockfile.path().to_path_buf());
        let barrier = Arc::new(Barrier::new(4));

        let mut handles = vec![];

        for _ in 0..3 {
            let lockpath_clone = Arc::clone(&lockpath);
            let barrier_clone = Arc::clone(&barrier);

            let handle = thread::spawn(move || {
                let lock = assert_ok!(lockpath_clone.as_path().lock_shared(ShouldBlock::Yes));
                barrier_clone.wait();
                thread::sleep(Duration::from_millis(50));
                drop(lock);
            });

            handles.push(handle);
        }

        barrier.wait();
        thread::sleep(Duration::from_millis(10));

        let result = lockpath.as_path().lock(ShouldBlock::No);
        assert_err!(&result);
        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::WouldBlock);

        for handle in handles {
            handle.join().expect("thread should not panic");
        }

        assert_ok!(lockpath.as_path().lock(ShouldBlock::Yes));
    }
}
