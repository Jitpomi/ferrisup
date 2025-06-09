// Template renderer for transforming templates with variable substitution
use handlebars::Handlebars;
use serde_json::Value;
use std::path::Path;
use std::fs;
use anyhow::{Result, anyhow};

/// Render a template file with variables
///
/// Takes a template file path and a set of variables, and returns the rendered content
/// with all variables substituted.
///
/// # Arguments
///
/// * `template_path` - Path to the template file
/// * `variables` - Variables to use for substitution
///
/// # Returns
///
/// The rendered content as a string
pub fn render_template(template_path: &Path, variables: &Value) -> Result<String> {
    // Read the template file
    if !template_path.exists() {
        return Err(anyhow!("Template file not found: {}", template_path.display()));
    }
    
    let template_content = fs::read_to_string(template_path)?;
    
    // Create Handlebars instance
    let mut handlebars = Handlebars::new();
    handlebars.register_escape_fn(handlebars::no_escape);
    
    // Render the template
    let rendered = handlebars.render_template(&template_content, variables)?;
    
    Ok(rendered)
}

/// Render a template string with variables
///
/// Takes a template string and a set of variables, and returns the rendered content
/// with all variables substituted.
///
/// # Arguments
///
/// * `template_string` - The template string
/// * `variables` - Variables to use for substitution
///
/// # Returns
///
/// The rendered content as a string
pub fn render_template_string(template_string: &str, variables: &Value) -> Result<String> {
    // Create Handlebars instance
    let mut handlebars = Handlebars::new();
    handlebars.register_escape_fn(handlebars::no_escape);
    
    // Render the template
    let rendered = handlebars.render_template(template_string, variables)?;
    
    Ok(rendered)
}

/// Register helpers with the Handlebars instance
///
/// This function registers custom helpers for use in templates
///
/// # Arguments
///
/// * `handlebars` - The Handlebars instance to register helpers with
pub fn register_helpers(handlebars: &mut Handlebars) {
    // Add any custom helpers here
    handlebars.register_helper(
        "lowercase",
        Box::new(|h: &handlebars::Helper, _: &handlebars::Handlebars, _: &handlebars::Context, _: &mut handlebars::RenderContext, out: &mut dyn handlebars::Output| -> handlebars::HelperResult {
            let param = h.param(0).ok_or_else(|| handlebars::RenderError::new("Missing parameter"))?;
            if let Some(s) = param.value().as_str() {
                out.write(&s.to_lowercase())?;
            } else {
                out.write(&param.value().to_string().to_lowercase())?;
            }
            Ok(())
        }),
    );
    
    handlebars.register_helper(
        "uppercase",
        Box::new(|h: &handlebars::Helper, _: &handlebars::Handlebars, _: &handlebars::Context, _: &mut handlebars::RenderContext, out: &mut dyn handlebars::Output| -> handlebars::HelperResult {
            let param = h.param(0).ok_or_else(|| handlebars::RenderError::new("Missing parameter"))?;
            if let Some(s) = param.value().as_str() {
                out.write(&s.to_uppercase())?;
            } else {
                out.write(&param.value().to_string().to_uppercase())?;
            }
            Ok(())
        }),
    );
    
    handlebars.register_helper(
        "pascal_case",
        Box::new(|h: &handlebars::Helper, _: &handlebars::Handlebars, _: &handlebars::Context, _: &mut handlebars::RenderContext, out: &mut dyn handlebars::Output| -> handlebars::HelperResult {
            let param = h.param(0).ok_or_else(|| handlebars::RenderError::new("Missing parameter"))?;
            let value_str = if let Some(s) = param.value().as_str() {
                s.to_string()
            } else {
                param.value().to_string()
            };
            
            let mut result = String::new();
            let mut capitalize_next = true;
            
            for c in value_str.chars() {
                if c.is_alphanumeric() {
                    if capitalize_next {
                        result.push(c.to_ascii_uppercase());
                        capitalize_next = false;
                    } else {
                        result.push(c);
                    }
                } else {
                    capitalize_next = true;
                }
            }
            
            out.write(&result)?;
            Ok(())
        }),
    );
    
    handlebars.register_helper(
        "snake_case",
        Box::new(|h: &handlebars::Helper, _: &handlebars::Handlebars, _: &handlebars::Context, _: &mut handlebars::RenderContext, out: &mut dyn handlebars::Output| -> handlebars::HelperResult {
            let param = h.param(0).ok_or_else(|| handlebars::RenderError::new("Missing parameter"))?;
            let value_str = if let Some(s) = param.value().as_str() {
                s.to_string()
            } else {
                param.value().to_string()
            };
            
            let mut result = String::new();
            let mut last_was_underscore = false;
            
            for c in value_str.chars() {
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
            
            out.write(&result)?;
            Ok(())
        }),
    );
}
