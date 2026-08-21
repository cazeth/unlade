//! Reads the per-crate files of a crates.io dump into [`unlade_core`] types.
//!
//! [`parse_crates`] returns the identity map, names, and update dates read from
//! `crates.csv`. Each component is addressed by the same
//! [`CrateIndex`](unlade_core::CrateIndex).
//! [`parse_downloads`] returns counts addressed by that same index.

#![warn(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::must_use_candidate)]

mod columns;
mod date_time;
mod downloads;
mod error;
mod parse;
mod row;

pub use crate::date_time::InvalidDateTime;
pub use crate::downloads::parse_downloads;
pub use crate::error::Error;
pub use crate::parse::{ParsedCrates, parse_crates};
