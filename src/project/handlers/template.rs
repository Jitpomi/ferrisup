// Template project handler implementation
use std::path::Path;
use serde_json::Value;
use handlebars::Handlebars;
use crate::project::handlers::traits::ProjectHandler;
use crate::core::Result;

/// Handler for template-based project generation
///
/// This handler is responsible for managing projects that are created using templates,
/// such as data science projects, server applications, etc. It delegates to the template
/// manager for template application and handles retrieving the correct next steps.
pub struct TemplateProjectHandler {
    /// Name of the template category
    name: String,
    
    /// Description of the template category
    description: String,
    
    /// Templates that this handler can handle
    templates: Vec<String>,
}

impl TemplateProjectHandler {
    /// Create a new template project handler
    pub fn new(name: &str, description: &str, templates: Vec<String>) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            templates,
        }
    }
}

impl ProjectHandler for TemplateProjectHandler {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn description(&self) -> &str {
        &self.description
    }
    
    fn can_handle(&self, template_name: &str, _variables: &Value) -> bool {
        self.templates.contains(&template_name.to_string())
    }
    
    fn initialize_project(&self, project_name: &str, target_dir: &Path, variables: &Value) -> Result<()> {
        // Use the template_manager to apply the template
        println!("📝 Creating new {} project using {} template...", project_name, self.name);
        
        // Get the template name from the variables
        let template_name = if let Some(template) = variables.get("template").and_then(|t| t.as_str()) {
            template
        } else {
            return Err("No template specified".into());
        };
        
        // Apply the template - this will be updated to use the new template manager module path
        crate::project::templates::apply_template(
            template_name, 
            target_dir, 
            project_name, 
            Some(variables.clone())
        )?;
        
        println!("✅ {} project created successfully!", project_name);
        Ok(())
    }
    
    fn get_next_steps(&self, project_name: &str, variables: &Value) -> Vec<String> {
        // Get template-based next steps
        if let Some(template) = variables.get("template").and_then(|t| t.as_str()) {
            // First, try to find next steps from the JSON file in the project directory
            // This is generated by post hooks and has highest priority
            let project_dir = Path::new(project_name);
            let next_steps_file = project_dir.join(".ferrisup_next_steps.json");
            
            if next_steps_file.exists() {
                if let Ok(content) = std::fs::read_to_string(&next_steps_file) {
                    if let Ok(json) = serde_json::from_str::<Value>(&content) {
                        if let Some(steps) = json.get("next_steps").and_then(|s| s.as_array()) {
                            let next_steps: Vec<String> = steps
                                .iter()
                                .filter_map(|s| s.as_str().map(|s| s.to_string()))
                                .collect();
                            
                            if !next_steps.is_empty() {
                                // Delete the file after reading
                                let _ = std::fs::remove_file(&next_steps_file);
                                return next_steps;
                            }
                        }
                    }
                }
            }
            
            // Second, try to get next steps from template.json with variable substitution
            if let Ok(template_config) = crate::project::templates::get_template_config(template) {
                if let Some(next_steps) = template_config.get("next_steps") {
                    // Handle array of steps (with variable substitution)
                    if let Some(steps) = next_steps.as_array() {
                        // Create a Handlebars instance for rendering
                        let mut handlebars = Handlebars::new();
                        handlebars.register_escape_fn(handlebars::no_escape);
                        
                        let mut result = Vec::new();
                        
                        for step in steps {
                            if let Some(step_str) = step.as_str() {
                                // Render template with variables
                                match handlebars.render_template(step_str, variables) {
                                    Ok(rendered) => {
                                        // Also replace {{project_name}} directly, as it might not be in variables
                                        let final_step = rendered.replace("{{project_name}}", project_name);
                                        result.push(final_step);
                                    },
                                    Err(_) => {
                                        // Fallback to direct replacement
                                        let step_text = step_str.replace("{{project_name}}", project_name);
                                        result.push(step_text);
                                    }
                                }
                            }
                        }
                        
                        if !result.is_empty() {
                            return result;
                        }
                    }
                    
                    // Handle object with conditional steps
                    if let Some(steps_obj) = next_steps.as_object() {
                        // Check for data_format-specific steps (important for Parquet support)
                        if let Some(data_format) = variables.get("data_format").and_then(|f| f.as_str()) {
                            if let Some(format_steps) = steps_obj.get(data_format).and_then(|s| s.as_array()) {
                                let mut result = Vec::new();
                                
                                for step in format_steps {
                                    if let Some(step_str) = step.as_str() {
                                        let step_text = step_str.replace("{{project_name}}", project_name);
                                        result.push(step_text);
                                    }
                                }
                                
                                if !result.is_empty() {
                                    return result;
                                }
                            }
                        }
                    }
                }
            }
            
            // Finally, try to use the template_manager's get_template_next_steps function
            if let Some(next_steps) = crate::project::templates::get_template_next_steps(
                template, 
                project_name, 
                Some(variables.clone())
            ) {
                return next_steps;
            }
        }
        
        // Default next steps if none found
        vec![
            format!("🚀 Navigate to your project: cd {}", project_name),
            "📝 Review the generated code".to_string(),
            "🔧 Build the project: cargo build".to_string(),
            "▶️ Run the project: cargo run".to_string(),
        ]
    }
}
