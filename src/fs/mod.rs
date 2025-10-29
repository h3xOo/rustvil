//! Filesystem utilities and extensions.
//!
//! ## Extensions traits
//!
//! Most interesting one is [`PathExt`].
//! Most of it are [`std::fs`] wrappers, changing from functional to OOP style, but there are some
//! interesting methods.
//!
//! ```rust,no_run
//! # use rustvil::fs::*;
//! # use std::path::Path;
//! # fn get_path() -> ! { loop {} }
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let path: &Path = get_path();
//! // Now you can do extra things like:
//! let _file = path.touch()?; // Creates file and its parent directories.
//! path.rm()?;
//! path.mkdir(MkdirOptions::WithParents)?;
//!
//! // You can also lock the file, to prevent races (even across different processes)
//! let _guard = path.lock(ShouldBlock::Yes)?;
//! // ...
//! drop(_guard);
//! // Cleanup created files.
//! path.rmtree()?;
//! # Ok(())
//! # }
//! ```

mod path_ext;
pub use path_ext::*;
