//! Bindings and wrappers to Node.js built-in modules.
//!
//! This crate is designed for internal use of **CodeCursor**. It may not cover
//! all the Node.js APIs, and it's also not the primary focus of this crate.

pub mod bindings;
#[cfg(feature = "futures")]
pub mod futures;
#[cfg(feature = "http_client")]
pub mod http_client;
pub mod macros;

pub mod prelude {
    pub use crate::bindings::{console, Buffer};
    pub use crate::{closure, closure_once};
}
