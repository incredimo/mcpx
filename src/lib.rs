//! # MCPX: Model Context Protocol SDK for Rust
//!
//! `mcpx` is a comprehensive Rust implementation of the Model Context Protocol (MCP),
//! designed to be robust, performant, and easy to use for both client and server applications.
//!
//! This library provides a complete implementation of the MCP specification (2025-03-26),
//! supporting all the standard message types, capabilities, and flows defined in the protocol.
//!
//! ## Features
//!
//! - Complete implementation of the MCP specification
//! - Asynchronous API using Tokio
//! - Type-safe request and response handling
//! - Support for both client and server roles
//! - Built-in connection management
//! - Comprehensive error handling
//! - Extensible architecture

pub mod client;
pub mod protocol;
pub mod server;
pub mod transport;
pub mod utils;
pub mod error;

// Re-export commonly used types for convenience
pub use client::Client;
pub use client::ClientBuilder;
pub use server::Server;
pub use server::ServerBuilder;
pub use protocol::Implementation;
pub use error::Error;
