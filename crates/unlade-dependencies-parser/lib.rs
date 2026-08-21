//! Count reverse dependencies in a crates.io database dump.
//!
//! This crate answers “how many other crates depend on this crate?” using the
//! `versions.csv` and `dependencies.csv` files from an extracted
//! [crates.io database dump](https://crates.io/data-access). It does not load
//! the crates being counted itself. Callers first load `crates.csv` into an
//! [`unlade_core::CrateIdMap`] value, then pass that map to [`count_dependents`].
//!
//! The returned [`unlade_core::Dependents`] uses the same
//! [`unlade_core::CrateIndex`] values as the input crates. This lets callers
//! use one index to retrieve a crate's name, identifier, update date, and
//! dependent count.
//!
//! # Example
//!
//! ```no_run
//! use std::error::Error;
//! use std::path::Path;
//! use unlade_crates_parser::parse_crates;
//! use unlade_dependencies_parser::count_dependents;
//!
//! fn main() -> Result<(), Box<dyn Error>> {
//!     // `data` is the directory inside an extracted crates.io dump.
//!     let data = Path::new("db-dump/data");
//!     let crates = parse_crates(&data.join("crates.csv"))?;
//!     let dependents = count_dependents(
//!         &data.join("versions.csv"),
//!         &data.join("dependencies.csv"),
//!         &crates.ids,
//!     )?;
//!
//!     for (index, count) in dependents.iter() {
//!         println!("{}: {count}", &crates.names[index]);
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! # What is counted
//!
//! For each crate in `versions.csv`, the greatest published version by
//! semantic-version precedence is selected. That version contributes at most
//! one dependent to each crate it names as an ordinary dependency, even when
//! the dependency appears more than once for different targets. Optional
//! dependencies count; build and development dependencies do not.
//!
//! Yanked versions are currently eligible to be selected. Crates referenced by
//! `dependencies.csv` but absent from the supplied [`unlade_core::CrateIdMap`] are
//! ignored. A supplied crate with no dependents receives a count of zero.
//!
//! # Performance and failures
//!
//! Both CSV files are streamed, but the parser retains the greatest-precedence
//! version of every crate and the dependency edges it has already counted. On a
//! complete crates.io dump, `versions.csv` and `dependencies.csv` are large, so this
//! operation can take substantially longer and use more memory than parsing
//! `crates.csv` alone.
//!
//! [`count_dependents`] returns [`Error`] when a required file cannot be opened
//! or read, a required column is missing, or a required field is malformed.

#![warn(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

mod dependencies;
mod error;
mod parse;
mod versions;

pub use crate::error::Error;
pub use crate::parse::count_dependents;
