//! Contains the [`Version`] struct.
//!
//! See the struct level documentation [`Version`].

use std::fmt::{Display, LowerHex, UpperHex};

use ash::vk;

/// Convenience wrapper around a Vulkan-formatted version.
///
/// For more information, see [`vk::make_api_version()`].
///
/// # Version compatibility
/// Version `a` is deemed compatible to version `b`, if
/// - `a.variant() == b.variant()` and
/// - `a.major() == b.major()` and
/// - `a.minor() >= b.minor()`
///
/// In other words, `variant` and `major` numbers denote backwards-incompatible changes,
/// `minor` numbers denote backwards-compatible changes and
/// `patch` numbers denote bug fixes that do not change the intended behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version(u32);

impl Version {
    /// Vulkan 1.0 as [`Version`]
    pub const VK10: Version = Version::from_vk_version(vk::API_VERSION_1_0);
    /// Vulkan 1.1 as [`Version`]
    pub const VK11: Version = Version::from_vk_version(vk::API_VERSION_1_1);
    /// Vulkan 1.2 as [`Version`]
    pub const VK12: Version = Version::from_vk_version(vk::API_VERSION_1_2);
    /// Vulkan 1.3 as [`Version`]
    pub const VK13: Version = Version::from_vk_version(vk::API_VERSION_1_3);

    /// Creates a new [`Version`] with the given components.
    pub const fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self(vk::make_api_version(0, major, minor, patch))
    }

    /// Same as [`Version::new()`], except that a variant can be specified.
    pub const fn new_with_variant(variant: u32, major: u32, minor: u32, patch: u32) -> Self {
        Self(vk::make_api_version(variant, major, minor, patch))
    }

    /// Creates a new [`Version`] wrapping the given Vulkan-formatted version.
    pub const fn from_vk_version(vk: u32) -> Self {
        Self(vk)
    }

    /// Returns the `variant` number.
    pub const fn variant(self) -> u32 {
        vk::api_version_variant(self.0)
    }
    /// Returns the `major` number.
    pub const fn major(self) -> u32 {
        vk::api_version_major(self.0)
    }
    /// Returns the `minor` number.
    pub const fn minor(self) -> u32 {
        vk::api_version_minor(self.0)
    }
    /// Returns the `patch` number.
    pub const fn patch(self) -> u32 {
        vk::api_version_patch(self.0)
    }

    /// Returns the wrapped Vulkan-formatted version
    pub const fn into_vk_version(self) -> u32 {
        self.0
    }

    /// Checks whether `self` is compatible to the [`Version`] specified in `v`.
    ///
    /// For compatibility rules, see [`Version`].
    pub const fn compatible_with(self, v: Self) -> bool {
        self.variant() == v.variant() && self.major() == v.major() && self.minor() >= v.minor()
    }
}

impl Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.sign_plus() {
            write!(
                f,
                "{}.{}.{}.{}",
                self.variant(),
                self.major(),
                self.minor(),
                self.patch()
            )
        } else {
            write!(f, "{}.{}.{}", self.major(), self.minor(), self.patch())
        }
    }
}

impl LowerHex for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.sign_plus() {
            write!(
                f,
                "{:x}.{:x}.{:x}.{:x}",
                self.variant(),
                self.major(),
                self.minor(),
                self.patch()
            )
        } else {
            write!(
                f,
                "{:x}.{:x}.{:x}",
                self.major(),
                self.minor(),
                self.patch()
            )
        }
    }
}

impl UpperHex for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.sign_plus() {
            write!(
                f,
                "{:X}.{:X}.{:X}.{:X}",
                self.variant(),
                self.major(),
                self.minor(),
                self.patch()
            )
        } else {
            write!(
                f,
                "{:X}.{:X}.{:X}",
                self.major(),
                self.minor(),
                self.patch()
            )
        }
    }
}
