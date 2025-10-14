//! Extension trait for [`Path`] with additional filesystem operations.
//!
//! Provides [`PathExt`] trait with convenient methods for file/directory operations,
//! file locking, and cross-platform executable detection.

use std::{
    fs::{
        File, Metadata, OpenOptions, Permissions, ReadDir, canonicalize, copy, create_dir,
        create_dir_all, exists, hard_link, metadata, read, read_dir, read_link, read_to_string,
        remove_dir, remove_dir_all, remove_file, rename, set_permissions, symlink_metadata, write,
    },
    io::{self},
    ops::{Deref, DerefMut},
    path::{Path, PathBuf},
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

/// Options for controlling [`PathExt::mkdir`]
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

/// Whether [`PathExt::lock`]/[`PathExt::lock_shared`] should block current thread.
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
    /// [`PathExt::mkdir`] or [`OpenOptions::open`].
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

    /// Create directories at given [`Path`].
    ///
    /// # Returns
    /// [`Ok(())`](Ok) if created successfully, otherwise error, as reported by
    /// [`create_dir`], or [`create_dir_all`].
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

    /// Wrapper around [`std::fs::remove_dir`].
    fn rmdir(&self) -> io::Result<()>;

    /// Wrapper around [`std::fs::remove_dir_all`].
    fn rmtree(&self) -> io::Result<()>;

    /// Wrapper around [`std::fs::remove_file`].
    fn rm(&self) -> io::Result<()>;

    /// Locks exclusively `self`, creating file if needed.
    ///
    /// This is essentially [`self.touch()?`](PathExt::touch) followed by [`File::lock`]/[`File::try_lock`], with RAII bloat.
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
    /// let lock = path.lock(ShouldBlock::Yes)?;
    /// // Critical section...
    /// drop(lock);
    ///
    /// let lock = path.lock(ShouldBlock::No)?;
    /// drop(lock);
    /// # let _ = path.rm();
    /// # Ok(())
    /// # }
    /// ```
    fn lock(&self, should_block: ShouldBlock) -> io::Result<FileLockGuard>;

    /// Locks shared `self`, creating file if needed.
    ///
    /// This is essentially [`self.touch()?`](PathExt::touch) followed by [`File::lock_shared`]/[`File::try_lock_shared`], with RAII bloat.
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
    /// let lock1 = path.lock_shared(ShouldBlock::Yes)?;
    /// let lock2 = path.lock_shared(ShouldBlock::No)?;
    /// drop(lock1);
    /// drop(lock2);
    /// # let _ = path.rm();
    /// # Ok(())
    /// # }
    /// ```
    fn lock_shared(&self, should_block: ShouldBlock) -> io::Result<FileLockGuard>;

    /// Returns `true` if path exists on a disk and points to an executable file.
    fn is_executable(&self) -> bool;

    /// Wrapper around [`std::fs::metadata`].
    fn metadata(&self) -> io::Result<Metadata>;

    /// Wrapper around [`std::fs::canonicalize`].
    fn canonicalize(&self) -> io::Result<PathBuf>;

    /// Wrapper around [`std::fs::copy`].
    fn copy_to(&self, to: impl AsRef<Path>) -> io::Result<u64>;

    /// Wrapper around [`std::fs::exists`].
    fn exists(&self) -> io::Result<bool>;

    /// Wrapper around [`std::fs::hard_link`].
    fn hard_link_to(&self, to: impl AsRef<Path>) -> io::Result<()>;

    /// Wrapper around [`std::fs::read`].
    fn read(&self) -> io::Result<Vec<u8>>;

    /// Wrapper around [`std::fs::read_dir`].
    fn read_dir(&self) -> io::Result<ReadDir>;

    /// Wrapper around [`std::fs::read_link`].
    fn read_link(&self) -> io::Result<PathBuf>;

    /// Wrapper around [`std::fs::read_to_string`].
    fn read_to_string(&self) -> io::Result<String>;

    /// Wrapper around [`std::fs::rename`].
    fn rename_to(&self, to: impl AsRef<Path>) -> io::Result<()>;

    /// Wrapper around [`std::fs::set_permissions`].
    fn set_permissions(&self, p: Permissions) -> io::Result<()>;

    /// Wrapper around [`std::fs::symlink_metadata`].
    fn symlink_metadata(&self) -> io::Result<Metadata>;

    /// Wrapper around [`std::fs::write`].
    fn write(&self, content: impl AsRef<[u8]>) -> io::Result<()>;
}

impl PathExt for Path {
    // FIXME: Take `OpenOptions` as a parameter?
    fn touch(&self) -> io::Result<File> {
        if let Some(parent) = self.parent() {
            parent.mkdir(MkdirOptions::WithParents)?;
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

    #[cfg(unix)]
    fn is_executable(&self) -> bool {
        use std::os::unix::prelude::*;
        self.metadata()
            .map(|metadata| {
                // Note: This should be the same as 0o111.
                const EXEC_MASK: u32 = (libc::S_IXUSR | libc::S_IXGRP | libc::S_IXOTH) as u32;
                const _: () = assert!(EXEC_MASK == 0o111, "bits mismatch");
                metadata.is_file() && metadata.permissions().mode() & EXEC_MASK != 0
            })
            .unwrap_or(false)
    }

    #[cfg(windows)]
    fn is_executable(&self) -> bool {
        self.is_file()
    }

    // TODO: Implement.
    #[cfg(not(any(unix, windows)))]
    fn is_executable(&self) -> bool {
        false
    }

    fn metadata(&self) -> io::Result<Metadata> {
        metadata(self)
    }

    fn canonicalize(&self) -> io::Result<PathBuf> {
        canonicalize(self)
    }

    fn copy_to(&self, to: impl AsRef<Path>) -> io::Result<u64> {
        copy(self, to)
    }

    fn exists(&self) -> io::Result<bool> {
        exists(self)
    }

    fn hard_link_to(&self, to: impl AsRef<Path>) -> io::Result<()> {
        hard_link(self, to)
    }

    fn read(&self) -> io::Result<Vec<u8>> {
        read(self)
    }

    fn read_dir(&self) -> io::Result<ReadDir> {
        read_dir(self)
    }

    fn read_link(&self) -> io::Result<PathBuf> {
        read_link(self)
    }

    fn read_to_string(&self) -> io::Result<String> {
        read_to_string(self)
    }

    fn rename_to(&self, to: impl AsRef<Path>) -> io::Result<()> {
        rename(self, to)
    }

    fn set_permissions(&self, permissions: Permissions) -> io::Result<()> {
        set_permissions(self, permissions)
    }

    fn symlink_metadata(&self) -> io::Result<Metadata> {
        symlink_metadata(self)
    }

    fn write(&self, contents: impl AsRef<[u8]>) -> io::Result<()> {
        write(self, contents)
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
