// Variables handling for template customization
use anyhow::Result;
use dialoguer::{Input, Select, Confirm};
use serde_json::{Value, json, Map};
use shared::to_pascal_case;

/// Process variables for a template
///
/// This function takes template configuration and interactive mode flag,
/// and generates a complete set of variables for template rendering.
///
/// # Arguments
///
/// * `template_config` - Template configuration with variable definitions
/// * `project_name` - Name of the project being created
/// * `interactive` - Whether to prompt the user for input
/// * `initial_variables` - Initial variables to use (from command line)
///
/// # Returns
///
/// A JSON Value containing all variables for template rendering
pub fn process_variables(
    template_config: &Value,
    project_name: &str,
    interactive: bool,
    initial_variables: Option<Value>,
) -> Result<Value> {
    // Create variables map with default values
    let mut variables = Map::new();
    
    // Add project name and derived values
    variables.insert("project_name".to_string(), json!(project_name));
    variables.insert("project_name_snake_case".to_string(), json!(to_snake_case(project_name)));
    variables.insert("project_name_pascal_case".to_string(), json!(to_pascal_case(project_name)));
    
    // Copy initial variables if provided
    if let Some(init_vars) = initial_variables {
        if let Some(obj) = init_vars.as_object() {
            for (key, value) in obj {
                variables.insert(key.clone(), value.clone());
            }
        }
    }
    
    // Get variables from template config
    if let Some(vars_config) = template_config.get("variables").and_then(|v| v.as_array()) {
        for var_config in vars_config {
            if let Some(var_obj) = var_config.as_object() {
                let name = var_obj.get("name").and_then(|n| n.as_str()).unwrap_or("");
                let prompt = var_obj.get("prompt").and_then(|p| p.as_str()).unwrap_or(name);
                let var_type = var_obj.get("type").and_then(|t| t.as_str()).unwrap_or("string");
                
                // Skip if already provided in initial variables
                if variables.contains_key(name) {
                    continue;
                }
                
                // Process variable based on type
                let value = match var_type {
                    "select" => {
                        if let Some(options) = var_obj.get("options").and_then(|o| o.as_array()) {
                            let options_str: Vec<String> = options
                                .iter()
                                .filter_map(|o| o.as_str().map(|s| s.to_string()))
                                .collect();
                            
                            // For the data_format variable, ensure we have the special mapping for Parquet
                            if name == "data_format" && options_str.contains(&"parquet".to_string()) {
                                let selected = if interactive {
                                    let selection = Select::new()
                                        .with_prompt(prompt)
                                        .items(&options_str)
                                        .default(0)
                                        .interact()?;
                                    options_str[selection].clone()
                                } else {
                                    options_str[0].clone()
                                };
                                
                                json!(selected)
                            } else {
                                let selected = if interactive {
                                    let selection = Select::new()
                                        .with_prompt(prompt)
                                        .items(&options_str)
                                        .default(0)
                                        .interact()?;
                                    options_str[selection].clone()
                                } else {
                                    options_str[0].clone()
                                };
                                
                                json!(selected)
                            }
                        } else {
                            json!("")
                        }
                    },
                    "boolean" => {
                        let default = var_obj.get("default").and_then(|d| d.as_bool()).unwrap_or(false);
                        
                        if interactive {
                            let selected = Confirm::new()
                                .with_prompt(prompt)
                                .default(default)
                                .interact()?;
                            
                            json!(selected)
                        } else {
                            json!(default)
                        }
                    },
                    _ => { // string or other
                        let default = var_obj.get("default").and_then(|d| d.as_str()).unwrap_or("");
                        
                        if interactive {
                            let input: String = Input::new()
                                .with_prompt(prompt)
                                .default(default.to_string())
                                .interact_text()?;
                            
                            json!(input)
                        } else {
                            json!(default)
                        }
                    }
                };
                
                variables.insert(name.to_string(), value);
            }
        }
    }
    
    // Special handling for conditional variables
    if let Some(data_source) = variables.get("data_source").and_then(|s| s.as_str()) {
        // If data_source is selected and data_format is not yet set
        if !variables.contains_key("data_format") {
            let data_format = match data_source {
                "csv" => "csv",
                "json" => "json",
                "parquet" => "parquet",
                _ => "csv"  // default
            };
            variables.insert("data_format".to_string(), json!(data_format));
        }
    }
    
    Ok(Value::Object(variables))
}

/// Convert a string to snake_case
fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    let mut last_was_underscore = false;
    
    for c in s.chars() {
        if c.is_alphanumeric() {
            result.push(c.to_ascii_lowercase());
            last_was_underscore = false;
        } else if !last_was_underscore {
            result.push('_');
            last_was_underscore = true;
        }
    }
    
    // Remove trailing underscore if any
    if result.ends_with('_') {
        result.pop();
    }
    
    result
}
