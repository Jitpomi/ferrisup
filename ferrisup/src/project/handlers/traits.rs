// Core trait that all project handlers must implement
use std::path::Path;
use serde_json::Value;
use crate::core::Result;

/// Project handler interface that defines the contract for all types of project handlers.
/// 
/// This is the core interface that enables separation between CLI-based and template-based
/// project creation, ensuring changes to one don't affect the other.
pub trait ProjectHandler {
    /// Get the name of this handler
    fn name(&self) -> &str;
    
    /// Get a description of this handler
    fn description(&self) -> &str;
    
    /// Check if this handler should be used for the given template
    /// 
    /// This method determines whether the handler can handle the specified template
    /// and variables. It's used by the registry to find the appropriate handler.
    fn can_handle(&self, template_name: &str, variables: &Value) -> bool;
    
    /// Initialize a project using this handler
    /// 
    /// This is where the actual project generation happens. For CLI handlers,
    /// it will invoke the external CLI tool. For template handlers, it will
    /// apply the template.
    fn initialize_project(&self, project_name: &str, target_dir: &Path, variables: &Value) -> Result<()>;
    
    /// Get next steps for a project created with this handler
    /// 
    /// This provides guidance to users after project creation. The steps may
    /// vary based on the project type, selected options, etc.
    fn get_next_steps(&self, project_name: &str, variables: &Value) -> Vec<String>;
}
