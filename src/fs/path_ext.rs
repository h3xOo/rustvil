use std::{
    fs::{File, OpenOptions, create_dir, create_dir_all, remove_dir, remove_dir_all, remove_file},
    path::Path,
};

/// Options for controlling `PathExt::mkdir`.
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
    impl<T> Sealed for T where T: AsRef<Path> {}
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
    fn touch(&self) -> Result<File, std::io::Error>;

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
    fn mkdir(&self, opts: MkdirOptions) -> Result<(), std::io::Error>;

    /// Wrapper around [`std::fs::remove_dir`](std::fs::remove_dir).
    fn rmdir(&self) -> Result<(), std::io::Error>;

    /// Wrapper around [`std::fs::remove_dir_all`](std::fs::remove_dir_all).
    fn rmtree(&self) -> Result<(), std::io::Error>;

    /// Wrapper around [`std::fs::remove_file`](std::fs::remove_file).
    fn rm(&self) -> Result<(), std::io::Error>;
}

impl<T: AsRef<Path>> PathExt for T {
    // FIXME: Take OpenOptions as a parameter?
    fn touch(&self) -> Result<File, std::io::Error> {
        let path = self.as_ref();
        fn inner(p: &Path) -> Result<File, std::io::Error> {
            if let Some(parent) = p.parent() {
                mkdir_impl(parent, MkdirOptions::WithParents)?;
            }
            OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .truncate(false)
                .open(p)
        }
        inner(path)
    }

    fn mkdir(&self, opts: MkdirOptions) -> Result<(), std::io::Error> {
        let path = self.as_ref();
        mkdir_impl(path, opts)
    }

    fn rmdir(&self) -> Result<(), std::io::Error> {
        remove_dir(self)
    }

    fn rmtree(&self) -> Result<(), std::io::Error> {
        remove_dir_all(self)
    }

    fn rm(&self) -> Result<(), std::io::Error> {
        remove_file(self)
    }
}

fn mkdir_impl(path: &Path, opts: MkdirOptions) -> Result<(), std::io::Error> {
    match opts {
        MkdirOptions::WithoutParents => create_dir(path),
        MkdirOptions::WithParents => create_dir_all(path),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use claim::{assert_err, assert_ok};
    use tempfile::tempdir;

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
}
