use anyhow::Result;
use std::path::Path;
use serde_json::Value;

/// Core trait that all project handlers must implement
pub trait ProjectHandler {
    /// Get the name of this handler
    fn name(&self) -> &str;
    
    /// Get a description of this handler
    fn description(&self) -> &str;
    
    /// Check if this handler should be used for the given template
    fn can_handle(&self, template_name: &str, variables: &Value) -> bool;
    
    /// Initialize a project using this handler
    fn initialize_project(&self, project_name: &str, target_dir: &Path, variables: &Value) -> Result<()>;
    
    /// Get next steps for a project created with this handler
    fn get_next_steps(&self, project_name: &str, variables: &Value) -> Vec<String>;
}
