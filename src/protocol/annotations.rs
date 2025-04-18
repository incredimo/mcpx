//! Annotations types for the MCP protocol

use serde::{Deserialize, Serialize};
use super::Role;

/// Optional annotations for the client. The client can use annotations to inform how objects are used or displayed.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Annotations {
    /// Describes who the intended customer of this object or data is.
    ///
    /// It can include multiple entries to indicate content useful for multiple audiences (e.g., `["user", "assistant"]`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audience: Option<Vec<Role>>,

    /// Describes how important this data is for operating the server.
    ///
    /// A value of 1 means "most important," and indicates that the data is
    /// effectively required, while 0 means "least important," and indicates that
    /// the data is entirely optional.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<f64>,
}

impl Annotations {
    /// Create new empty annotations
    pub fn new() -> Self {
        Self::default()
    }

    /// Create new annotations with audience
    pub fn with_audience(audience: Vec<Role>) -> Self {
        Self {
            audience: Some(audience),
            priority: None,
        }
    }

    /// Create new annotations with priority
    pub fn with_priority(priority: f64) -> Self {
        // Clamp priority between 0 and 1
        let priority = priority.max(0.0).min(1.0);
        
        Self {
            audience: None,
            priority: Some(priority),
        }
    }

    /// Create new annotations with audience and priority
    pub fn with_audience_and_priority(audience: Vec<Role>, priority: f64) -> Self {
        // Clamp priority between 0 and 1
        let priority = priority.max(0.0).min(1.0);
        
        Self {
            audience: Some(audience),
            priority: Some(priority),
        }
    }

    /// Add an audience role to the annotations
    pub fn add_audience(&mut self, role: Role) {
        if let Some(audience) = &mut self.audience {
            audience.push(role);
        } else {
            self.audience = Some(vec![role]);
        }
    }

    /// Set the priority of the annotations
    pub fn set_priority(&mut self, priority: f64) {
        // Clamp priority between 0 and 1
        let priority = priority.max(0.0).min(1.0);
        self.priority = Some(priority);
    }
}
