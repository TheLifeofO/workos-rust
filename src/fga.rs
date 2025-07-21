//! A module for interacting with the WorkOS Fine-Grained Authorization API.
//!
//! [WorkOS Docs: Fine-Grained Authorization](https://workos.com/docs/reference/fga)

mod operations;
mod types;

pub use operations::*;
pub use types::*;

use crate::WorkOs;

/// Fine-Grained Authorization.
///
/// [WorkOS Docs: Fine-Grained Authorization](https://workos.com/docs/reference/fga)
pub struct Fga<'a> {
    workos: &'a WorkOs,
}

impl<'a> Fga<'a> {
    /// Returns a new [`Fga`] instance for the provided WorkOS client.
    pub fn new(workos: &'a WorkOs) -> Self {
        Self { workos }
    }
}
