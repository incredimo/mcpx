//! JSON manipulation utilities
//!
//! This module provides functions for working with JSON data.

use serde_json::{Value, Map};
use crate::error::Error;

/// Merge two JSON objects
///
/// The `target` object will be modified to include values from the `source` object.
/// If a key exists in both objects, the value from `source` will override the value in `target`.
pub fn merge_json_objects(target: &mut Value, source: &Value) {
    if let (Value::Object(target_map), Value::Object(source_map)) = (target, source) {
        for (key, value) in source_map {
            if !target_map.contains_key(key) {
                target_map.insert(key.clone(), value.clone());
            } else {
                let target_value = target_map.get_mut(key).unwrap();
                if target_value.is_object() && value.is_object() {
                    merge_json_objects(target_value, value);
                } else {
                    *target_value = value.clone();
                }
            }
        }
    }
}

/// Get a value from a JSON object using a path
///
/// The path is a string of keys separated by dots, e.g., "foo.bar.baz".
pub fn json_path_get<'a>(json: &'a Value, path: &str) -> Option<&'a Value> {
    let mut current = json;
    for key in path.split('.') {
        match current {
            Value::Object(map) => {
                if let Some(value) = map.get(key) {
                    current = value;
                } else {
                    return None;
                }
            }
            Value::Array(array) => {
                if let Ok(index) = key.parse::<usize>() {
                    if let Some(value) = array.get(index) {
                        current = value;
                    } else {
                        return None;
                    }
                } else {
                    return None;
                }
            }
            _ => return None,
        }
    }
    Some(current)
}

/// Set a value in a JSON object using a path
///
/// The path is a string of keys separated by dots, e.g., "foo.bar.baz".
/// If any part of the path doesn't exist, it will be created.
pub fn json_path_set(json: &mut Value, path: &str, value: Value) -> Result<(), Error> {
    let parts: Vec<&str> = path.split('.').collect();
    if parts.is_empty() {
        return Err(Error::ParseError("Empty path".to_string()));
    }
    
    let mut current = json;
    for (i, key) in parts.iter().enumerate() {
        if i == parts.len() - 1 {
            // Last part of the path, set the value
            match current {
                Value::Object(map) => {
                    map.insert(key.to_string(), value);
                    return Ok(());
                }
                Value::Array(array) => {
                    if let Ok(index) = key.parse::<usize>() {
                        if index < array.len() {
                            array[index] = value;
                            return Ok(());
                        } else {
                            return Err(Error::ParseError(format!("Array index out of bounds: {}", index)));
                        }
                    } else {
                        return Err(Error::ParseError(format!("Invalid array index: {}", key)));
                    }
                }
                _ => return Err(Error::ParseError("Cannot set value on non-object or non-array".to_string())),
            }
        } else {
            // Intermediate part of the path
            match current {
                Value::Object(map) => {
                    if !map.contains_key(*key) {
                        // Create the path
                        let next_key = parts[i + 1];
                        let next = if next_key.parse::<usize>().is_ok() {
                            Value::Array(Vec::new())
                        } else {
                            Value::Object(Map::new())
                        };
                        map.insert(key.to_string(), next);
                    }
                    current = map.get_mut(*key).unwrap();
                }
                Value::Array(array) => {
                    if let Ok(index) = key.parse::<usize>() {
                        if index < array.len() {
                            current = &mut array[index];
                        } else {
                            return Err(Error::ParseError(format!("Array index out of bounds: {}", index)));
                        }
                    } else {
                        return Err(Error::ParseError(format!("Invalid array index: {}", key)));
                    }
                }
                _ => return Err(Error::ParseError("Cannot traverse non-object or non-array".to_string())),
            }
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_merge_json_objects() {
        let mut target = json!({
            "name": "John",
            "age": 30,
            "address": {
                "city": "New York"
            }
        });
        
        let source = json!({
            "age": 31,
            "address": {
                "street": "Broadway"
            },
            "phone": "123-456-7890"
        });
        
        merge_json_objects(&mut target, &source);
        
        assert_eq!(
            target,
            json!({
                "name": "John",
                "age": 31,
                "address": {
                    "city": "New York",
                    "street": "Broadway"
                },
                "phone": "123-456-7890"
            })
        );
    }

    #[test]
    fn test_json_path_get() {
        let json = json!({
            "user": {
                "name": "John",
                "contacts": [
                    { "type": "email", "value": "john@example.com" },
                    { "type": "phone", "value": "123-456-7890" }
                ]
            }
        });
        
        assert_eq!(json_path_get(&json, "user.name").unwrap(), &json!("John"));
        assert_eq!(
            json_path_get(&json, "user.contacts.0.value").unwrap(),
            &json!("john@example.com")
        );
        assert_eq!(json_path_get(&json, "user.age"), None);
    }

    #[test]
    fn test_json_path_set() {
        let mut json = json!({
            "user": {
                "name": "John",
                "contacts": [
                    { "type": "email", "value": "john@example.com" }
                ]
            }
        });
        
        json_path_set(&mut json, "user.age", json!(31)).unwrap();
        json_path_set(&mut json, "user.contacts.0.value", json!("new-email@example.com")).unwrap();
        json_path_set(&mut json, "user.settings.theme", json!("dark")).unwrap();
        
        assert_eq!(
            json,
            json!({
                "user": {
                    "name": "John",
                    "age": 31,
                    "contacts": [
                        { "type": "email", "value": "new-email@example.com" }
                    ],
                    "settings": {
                        "theme": "dark"
                    }
                }
            })
        );
    }
}
