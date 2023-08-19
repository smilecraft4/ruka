//! Ruka Prelude

pub use crate::error::Error;

/// Ruka's error result
pub type Result<T> = core::result::Result<T, Error>;

// Generic wrapper for newtype pattern
pub struct W<T>(pub T);
