pub mod backend;
pub mod cli;
pub mod client;
pub mod error;
pub mod mcp;
pub mod openapi;

#[cfg(all(target_os = "wasi", target_env = "p2"))]
pub mod wasm;

pub use error::{Error, Result};
