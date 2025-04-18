//! URI manipulation utilities
//!
//! This module provides functions for working with URIs and URI templates.

use url::Url;
use uri_template_system::{Template, Value, Values};
use std::collections::HashMap;
use crate::error::Error;

/// Check if a string is a valid URI
pub fn is_valid_uri(uri: &str) -> bool {
    Url::parse(uri).is_ok()
}

/// Parse a URI template with parameters
pub fn parse_uri_template(template: &str, params: HashMap<String, String>) -> Result<String, Error> {
    let uri_template = Template::parse(template)
        .map_err(|e| Error::ParseError(format!("Invalid URI template: {}", e)))?;

    let mut values = Values::default();
    for (key, value) in params {
        values = values.add(key, Value::item(value));
    }

    let expanded = uri_template.expand(&values)
        .map_err(|e| Error::ParseError(format!("Failed to expand URI template: {}", e)))?;

    Ok(expanded)
}

/// Extract path segments from a URI
pub fn uri_path_segments(uri: &str) -> Result<Vec<String>, Error> {
    let url = Url::parse(uri)
        .map_err(|e| Error::ParseError(format!("Invalid URI: {}", e)))?;

    let segments = url.path_segments()
        .map(|segments| segments.map(|s| s.to_string()).collect::<Vec<_>>())
        .unwrap_or_default();

    Ok(segments)
}

/// Join path segments into a URI
pub fn join_uri_paths(base: &str, path: &str) -> Result<String, Error> {
    let mut url = Url::parse(base)
        .map_err(|e| Error::ParseError(format!("Invalid base URI: {}", e)))?;

    // Normalize path to remove leading/trailing slashes
    let path = path.trim_start_matches('/').trim_end_matches('/');

    // Join paths
    let mut url_path = url.path().trim_end_matches('/').to_string();
    url_path.push('/');
    url_path.push_str(path);

    url.set_path(&url_path);

    Ok(url.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_uri() {
        assert!(is_valid_uri("http://example.com"));
        assert!(is_valid_uri("file:///path/to/file"));
        assert!(!is_valid_uri("invalid uri"));
    }

    #[test]
    fn test_parse_uri_template() {
        let template = "http://example.com/{path}/{id}";
        let mut params = HashMap::new();
        params.insert("path".to_string(), "users".to_string());
        params.insert("id".to_string(), "123".to_string());

        let result = parse_uri_template(template, params).unwrap();
        assert_eq!(result, "http://example.com/users/123");
    }

    #[test]
    fn test_uri_path_segments() {
        let uri = "http://example.com/foo/bar/baz";
        let segments = uri_path_segments(uri).unwrap();
        assert_eq!(segments, vec!["foo", "bar", "baz"]);
    }

    #[test]
    fn test_join_uri_paths() {
        let base = "http://example.com/api";
        let path = "users/123";
        let joined = join_uri_paths(base, path).unwrap();
        assert_eq!(joined, "http://example.com/api/users/123");
    }
}
