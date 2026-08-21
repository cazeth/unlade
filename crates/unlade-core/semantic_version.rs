//! Semantic versions of software releases.

use core::cmp::Ordering;
use core::fmt;
use core::str::FromStr;

/// A semantic version of a software release.
///
/// A semantic version has major, minor, and patch numbers and may also have
/// prerelease or build metadata. Semantic-version precedence ignores build
/// metadata, so use [`cmp_precedence`](Self::cmp_precedence) when deciding which
/// of two releases has greater precedence.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct SemanticVersion(semver::Version);

/// A string could not be parsed as a [`SemanticVersion`].
#[derive(Debug)]
pub struct ParseSemanticVersionError(semver::Error);

impl fmt::Display for ParseSemanticVersionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl std::error::Error for ParseSemanticVersionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.0)
    }
}

impl SemanticVersion {
    /// The semantic version, without prerelease or build metadata.
    pub fn new(major: u64, minor: u64, patch: u64) -> Self {
        Self(semver::Version::new(major, minor, patch))
    }

    /// The major version number.
    pub const fn major(&self) -> u64 {
        self.0.major
    }

    /// The minor version number.
    pub const fn minor(&self) -> u64 {
        self.0.minor
    }

    /// The patch version number.
    pub const fn patch(&self) -> u64 {
        self.0.patch
    }

    /// Whether this is a prerelease version.
    pub fn is_prerelease(&self) -> bool {
        !self.0.pre.is_empty()
    }

    /// Compares two versions according to semantic-version precedence.
    ///
    /// Build metadata is ignored, as required by semantic versioning.
    pub fn cmp_precedence(&self, other: &Self) -> Ordering {
        self.0.cmp_precedence(&other.0)
    }

    /// Returns whether this version has greater precedence than `other`.
    pub fn outranks(&self, other: &Self) -> bool {
        self.cmp_precedence(other) == Ordering::Greater
    }
}

impl FromStr for SemanticVersion {
    type Err = ParseSemanticVersionError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        text.parse().map(Self).map_err(ParseSemanticVersionError)
    }
}

impl fmt::Display for SemanticVersion {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn version(text: &str) -> SemanticVersion {
        text.parse().expect("version parses")
    }

    #[test]
    fn components_are_available() {
        let version = version("12.34.56");
        assert_eq!(version.major(), 12);
        assert_eq!(version.minor(), 34);
        assert_eq!(version.patch(), 56);
    }

    #[test]
    fn prereleases_are_identified() {
        assert!(version("1.0.0-alpha.1").is_prerelease());
        assert!(!version("1.0.0").is_prerelease());
    }

    #[test]
    fn versions_compare_by_semver_precedence() {
        assert!(version("1.10.0").outranks(&version("1.9.0")));
        assert!(version("2.0.0").outranks(&version("2.0.0-rc.1")));
    }

    #[test]
    fn build_metadata_does_not_affect_precedence() {
        assert_eq!(
            version("1.0.0+linux").cmp_precedence(&version("1.0.0+windows")),
            Ordering::Equal,
        );
    }

    #[test]
    fn display_preserves_the_semantic_version() {
        assert_eq!(
            version("1.2.3-alpha.1+linux").to_string(),
            "1.2.3-alpha.1+linux",
        );
    }

    #[test]
    fn malformed_versions_are_rejected() {
        assert!("not-a-version".parse::<SemanticVersion>().is_err());
    }
}
