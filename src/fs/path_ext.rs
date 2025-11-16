#[cfg(feature = "full-canonicalize")]
use std::path::PathBuf;

use std::{
    fs::{
        File, OpenOptions, Permissions, copy, create_dir, create_dir_all, hard_link, read,
        read_to_string, remove_dir, remove_dir_all, remove_file, rename, set_permissions, write,
    },
    io::{self},
    ops::{Deref, DerefMut},
    path::Path,
};

/// RAII guard, which calls [`(*self).unlock()`](std::fs::File::unlock) on drop.
#[derive(Debug)]
pub struct FileLockGuard {
    file: File,
}

impl Drop for FileLockGuard {
    fn drop(&mut self) {
        drop(self.file.unlock())
    }
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

/// Whether [`PathExt::lock`]/[`PathExt::lock_shared`] should block the current thread.
#[derive(Debug, Hash, Clone, Copy, PartialEq, Eq)]
pub enum ShouldBlock {
    No,
    Yes,
}

/// Extension trait for [`Path`] with additional filesystem operations.
///
/// Most of it are [`std::fs`] wrappers, changing from functional to OOP style, but there are some
/// interesting methods.
///
/// ```rust,no_run
/// # use rustvil::fs::*;
/// # use std::path::Path;
/// # fn get_path() -> ! { loop {} }
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let path: &Path = get_path();
/// // Now you can do extra things like:
/// let _file = path.touch()?; // Creates file and its parent directories.
/// path.rm()?;
/// path.mkdir(MkdirOptions::WithParents)?;
///
/// // You can also lock the file, to prevent races (even across different processes)
/// let _guard = path.lock(ShouldBlock::Yes)?;
/// // ...
/// drop(_guard);
/// // Cleanup created files.
/// path.rmtree()?;
/// # Ok(())
/// # }
pub trait PathExt: sealed::Sealed {
    /// Touch file and its parent directories.
    ///
    /// # Returns
    /// [`Ok(File)`](std::fs::File) if created successfully, otherwise error, as reported by
    /// [`PathExt::mkdir`] or [`OpenOptions::open`].
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use rustvil::fs::PathExt;
    /// # use std::path::PathBuf;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let buf = PathBuf::from("file.txt");
    /// let path = buf.as_path();
    /// let file = path.touch()?;
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
    /// Note that this function will return `Ok(())`, if [`create_dir`] returns `Err` with kind
    /// [`ErrorKind::AlreadyExists`](io::ErrorKind::AlreadyExists).
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use rustvil::fs::*;
    /// # use std::path::PathBuf;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let buf = PathBuf::from("a/b");
    /// let path = buf.as_path();
    /// path.mkdir(MkdirOptions::WithParents)?;
    /// # Ok(())
    /// # }
    /// ```
    fn mkdir(&self, opts: MkdirOptions) -> io::Result<()>;

    /// Locks exclusively `self`, creating file if needed.
    ///
    /// This is essentially [`self.touch()?`](PathExt::touch) followed by [`File::lock`]/[`File::try_lock`], with RAII bloat.
    ///
    /// # Returns
    /// [`Ok(FileLockGuard)`](FileLockGuard) on success.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use rustvil::fs::*;
    /// # use std::path::PathBuf;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let buf = PathBuf::from("lockfile.lock");
    /// let path = buf.as_path();
    ///
    /// let lock = path.lock(ShouldBlock::Yes)?;
    /// // Critical section...
    /// drop(lock);
    ///
    /// let lock = path.lock(ShouldBlock::No)?;
    /// drop(lock);
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
    /// ```rust,no_run
    /// # use rustvil::fs::*;
    /// # use std::path::PathBuf;
    /// # use std::path::Path;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let buf = PathBuf::from("lockfile.lock");
    /// let path = buf.as_path();
    ///
    /// let lock1 = path.lock_shared(ShouldBlock::Yes)?;
    /// let lock2 = path.lock_shared(ShouldBlock::No)?;
    /// drop(lock1);
    /// drop(lock2);
    /// # Ok(())
    /// # }
    /// ```
    fn lock_shared(&self, should_block: ShouldBlock) -> io::Result<FileLockGuard>;

    /// Canonicalize `self` fully: expand `~` into a `$HOME`, and resolve symlinks.
    ///
    /// Unlike [`std::fs::canonicalize`], this function __doesn't__ fail, if `self` points to
    /// non-existing file.
    ///
    /// Also, because of the current implementation, this function will fail, if `self` is not an
    /// `UTF-8` path.
    ///
    /// This function requires __full-canonicalize__ feature.
    #[cfg(feature = "full-canonicalize")]
    #[cfg_attr(docsrs, doc(cfg(feature = "full-canonicalize")))]
    fn full_canonicalize(&self) -> io::Result<PathBuf>;

    /// Returns `true` if path exists on a disk and points to an executable file.
    ///
    /// Current implementation only considers `unix` and `windows` cfg's, any other always returns
    /// `false`.
    fn is_executable(&self) -> bool;

    /// Wrapper around [`std::fs::copy`].
    fn copy_to(&self, to: impl AsRef<Path>) -> io::Result<u64>;

    /// Wrapper around [`std::fs::hard_link`].
    fn hard_link_to(&self, to: impl AsRef<Path>) -> io::Result<()>;

    /// Wrapper around [`std::fs::read`].
    fn read(&self) -> io::Result<Vec<u8>>;

    /// Wrapper around [`std::fs::read_to_string`].
    fn read_to_string(&self) -> io::Result<String>;

    /// Wrapper around [`std::fs::rename`].
    fn rename_to(&self, to: impl AsRef<Path>) -> io::Result<()>;

    /// Wrapper around [`std::fs::remove_file`].
    fn rm(&self) -> io::Result<()>;

    /// Wrapper around [`std::fs::remove_dir`].
    fn rmdir(&self) -> io::Result<()>;

    /// Wrapper around [`std::fs::remove_dir_all`].
    fn rmtree(&self) -> io::Result<()>;

    /// Wrapper around [`std::fs::set_permissions`].
    fn set_permissions(&self, permissions: Permissions) -> io::Result<()>;

    /// Wrapper around [`std::fs::write`].
    fn write(&self, contents: impl AsRef<[u8]>) -> io::Result<()>;
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
        let result = match opts {
            MkdirOptions::WithoutParents => create_dir(self),
            MkdirOptions::WithParents => create_dir_all(self),
        };
        match result {
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => Ok(()),
            _ => result,
        }
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
                #[allow(clippy::unnecessary_cast)] // On macOS those are u16, on Linux they are u32.
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

    fn copy_to(&self, to: impl AsRef<Path>) -> io::Result<u64> {
        copy(self, to)
    }

    fn hard_link_to(&self, to: impl AsRef<Path>) -> io::Result<()> {
        hard_link(self, to)
    }

    fn read(&self) -> io::Result<Vec<u8>> {
        read(self)
    }

    fn read_to_string(&self) -> io::Result<String> {
        read_to_string(self)
    }

    fn rename_to(&self, to: impl AsRef<Path>) -> io::Result<()> {
        rename(self, to)
    }

    fn rm(&self) -> io::Result<()> {
        remove_file(self)
    }

    fn rmdir(&self) -> io::Result<()> {
        remove_dir(self)
    }

    fn rmtree(&self) -> io::Result<()> {
        remove_dir_all(self)
    }

    fn set_permissions(&self, permissions: Permissions) -> io::Result<()> {
        set_permissions(self, permissions)
    }

    fn write(&self, contents: impl AsRef<[u8]>) -> io::Result<()> {
        write(self, contents)
    }

    #[cfg(feature = "full-canonicalize")]
    fn full_canonicalize(&self) -> io::Result<PathBuf> {
        use shellexpand::tilde;
        use soft_canonicalize::soft_canonicalize;
        let Some(as_str) = self.to_str() else {
            return Err(io::Error::other("path is not an UTF-8 string"));
        };
        let expanded = tilde(as_str);
        soft_canonicalize(expanded.into_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use claim::{assert_err, assert_ok};
    use tempfile::tempdir;

    use std::io::{Read, Write};
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
    fn multiple_touch() {
        let tmp = tempdir().expect("needed for tests");
        let mut new_file = tmp.path().to_path_buf();
        new_file.push("x");
        let mut file = new_file.touch().unwrap();
        file.write_all("test".as_bytes()).unwrap();
        let mut new_handle = new_file.touch().unwrap();
        let mut content = String::new();
        let read_bytes = new_handle.read_to_string(&mut content).unwrap();
        assert_eq!(read_bytes, 4);
        assert_eq!(content, "test");
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
    fn mkdir_doesnt_screw_paths() {
        let tmp = tempdir().expect("needed for tests");
        let mut new_file = tmp.path().to_path_buf();
        let mut copy = new_file.clone();
        let mut copy2 = new_file.clone();
        new_file.push("x");
        new_file.push("y");
        assert_ok!(new_file.mkdir(MkdirOptions::WithParents));
        assert_ok!(new_file.mkdir(MkdirOptions::WithoutParents));
        copy.push("x");
        copy.push("file");
        copy2.push("x");
        assert_ok!(copy.touch());
        assert_ok!(copy2.mkdir(MkdirOptions::WithParents));
        assert_ok!(copy2.mkdir(MkdirOptions::WithoutParents));
        assert!(copy.exists());
        assert!(copy2.exists());
        assert!(new_file.exists());
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
