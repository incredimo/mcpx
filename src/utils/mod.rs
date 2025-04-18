//! Utility functions and types for the MCP SDK
//!
//! This module contains utility functions and types used throughout the MCP SDK.

pub mod base64;
pub mod uri;
pub mod json;

// Re-export commonly used utilities
pub use self::base64::{encode_base64, decode_base64};
pub use self::uri::{is_valid_uri, parse_uri_template};
pub use self::json::{merge_json_objects, json_path_get, json_path_set};
