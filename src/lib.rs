#![cfg_attr(docsrs, feature(doc_cfg))]

//! Rustvil - A collection of various Rust utilities.
//!
//! Look at documentation of each submodule for more verbose documentation.
//!
//! ## Rationalize
//!
//! This crate was created as sort of _storage_ for various components I found myself implementing,
//! because they were missing from standard library.

pub mod config_files;
pub mod fs;
pub mod os;
pub mod signals;
