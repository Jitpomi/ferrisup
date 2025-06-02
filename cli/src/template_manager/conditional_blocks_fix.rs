use anyhow::Result;
use serde_json::Value;

/// Process conditional blocks in template content
pub fn process_conditional_blocks(content: &str, variables: &Value) -> Result<String> {
    let mut result = content.to_string();
    
    // Process cloud_provider conditionals if available
    if let Some(cloud_provider) = variables.get("cloud_provider").and_then(|p| p.as_str()) {
        let providers = ["aws", "gcp", "azure", "vercel", "netlify"];
        
        for provider in providers {
            process_conditional_for_variable(&mut result, "cloud_provider", provider, cloud_provider);
        }
    }
    
    // Return the processed content
    Ok(result)
}

/// Process conditionals for a specific variable and value
fn process_conditional_for_variable(content: &mut String, variable_name: &str, value: &str, selected_value: &str) {
    let start_tag = format!("{{{{#if (eq {} \"{}\")}}}}}", variable_name, value);
    let end_tag = "{{/if}}";
    
    // Find all blocks for this value
    let mut start_idx = 0;
    while let Some(block_start) = content[start_idx..].find(&start_tag) {
        let block_start = start_idx + block_start;
        
        // Find the matching end tag
        if let Some(block_end) = content[block_start..].find(end_tag) {
            let block_end = block_start + block_end + end_tag.len();
            
            // If this is the selected value, keep the content but remove the tags
            if value == selected_value {
                let content_start = block_start + start_tag.len();
                let content_end = block_end - end_tag.len();
                
                // Create a new string with the content but without the tags
                let new_content = format!(
                    "{}{}{}",
                    &content[0..block_start],
                    &content[content_start..content_end],
                    &content[block_end..]
                );
                
                *content = new_content;
                
                // Adjust the start index for the next search
                start_idx = block_start + (content_end - content_start);
            } else {
                // This is not the selected value, remove the entire block
                let new_content = format!(
                    "{}{}",
                    &content[0..block_start],
                    &content[block_end..]
                );
                
                *content = new_content;
                
                // Adjust the start index for the next search
                start_idx = block_start;
            }
        } else {
            // No matching end tag found, break the loop
            break;
        }
    }
}
