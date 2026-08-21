//! Types for storing information about crates from a crates.io database dump.
//! # Storing crate information
//!
//! A `CrateIndex` is used to look up information in the other types provided by
//! this crate. [`Names`], [`UpdateDates`], [`Downloads`], and [`Dependents`] each
//! store one kind of information. Entry 0 in any of those stores belongs to the
//! crate that `CrateIdMap` assigned index 0, entry 1 belongs to index 1, and so
//! on.
//!
//! The usual workflow is:
//!
//! 1. Pass each ID read from a dump to [`CrateIdMap::get_or_insert`].
//! 2. Use the returned `CrateIndex` as the crate's identity inside the program.
//! 3. Store and retrieve that crate's name, dates, and counts at the same index.
//!
//! The component stores are separate, so a program can create only the ones it
//! needs. A `CrateIndex` is meaningful only with the `CrateIdMap` and component
//! stores that were populated together.
//!
//! # Example
//!
//! ```
//! use unlade_core::{CrateId, CrateIdMap, Downloads, Names};
//!
//! let mut ids = CrateIdMap::new();
//! let mut names = Names::new();
//! let mut downloads = Downloads::new();
//!
//! // Allocate identities, then append each component in the same index order.
//! let serde = ids.get_or_insert(CrateId::new(1));
//! assert_eq!(names.push("serde"), serde);
//! assert_eq!(downloads.push(12_000_000), serde);
//!
//! let tokio = ids.get_or_insert(CrateId::new(8));
//! assert_eq!(names.push("tokio"), tokio);
//! assert_eq!(downloads.push(25_000_000), tokio);
//!
//! // Seeing an existing crates.io ID resolves to the same index.
//! assert_eq!(ids.get_or_insert(CrateId::new(1)), serde);
//! assert_eq!(ids.get(CrateId::new(8)), Some(tokio));
//! assert_eq!(ids.id(serde), Some(CrateId::new(1)));
//!
//! // The index addresses every component without another hash lookup.
//! assert_eq!(&names[serde], "serde");
//! assert_eq!(downloads[serde], 12_000_000);
//! ```
//!
//! These stores do not contain a `CrateIdMap` and do not validate one
//! another. Code constructing stores directly must append values in identity
//! index order, as above.
//!
//! The optional `serde` feature serializes the component stores and identity map.
//!
//! Times are `jiff::Timestamp`, re-exported here so consumers share the same
//! version of jiff as this crate.

#![warn(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::must_use_candidate)]

mod crate_id;
mod crate_id_map;
mod dependents;
mod downloads;
mod index;
mod names;
mod semantic_version;
mod store;
mod update_dates;

pub use crate::crate_id::CrateId;
pub use crate::crate_id_map::CrateIdMap;
pub use crate::dependents::Dependents;
pub use crate::downloads::Downloads;
pub use crate::index::CrateIndex;
pub use crate::names::Names;
pub use crate::semantic_version::ParseSemanticVersionError;
pub use crate::semantic_version::SemanticVersion;
pub use crate::update_dates::UpdateDates;
pub use jiff;
