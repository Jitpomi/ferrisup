use anyhow::Result;
use serde_json::Value;

/// Process conditional blocks in template content
pub fn process_conditional_blocks(content: &str, variables: &Value) -> Result<String> {
    let mut result = content.to_string();
    
    // Process cloud_provider conditionals
    if let Some(cloud_provider) = variables.get("cloud_provider").and_then(|p| p.as_str()) {
        result = process_variable_conditionals(&result, "cloud_provider", cloud_provider, &["aws", "gcp", "azure", "vercel", "netlify"]);
    }
    
    // Process data_source conditionals
    if let Some(data_source) = variables.get("data_source").and_then(|p| p.as_str()) {
        result = process_variable_conditionals(&result, "data_source", data_source, &["CSV files", "JSON data", "Parquet files"]);
    }
    
    // Process visualization conditionals
    if let Some(visualization) = variables.get("visualization").and_then(|p| p.as_str()) {
        result = process_variable_conditionals(&result, "visualization", visualization, &["yes", "no"]);
    }
    
    Ok(result)
}

/// Process conditionals for a specific variable
fn process_variable_conditionals(content: &str, variable_name: &str, selected_value: &str, possible_values: &[&str]) -> String {
    let mut result = content.to_string();
    
    for value in possible_values {
        let start_tag = format!("{{{{#if (eq {} \"{}\")}}}}}", variable_name, value);
        let end_tag = "{{/if}}";
        
        // Find all blocks for this value
        let mut start_idx = 0;
        while let Some(block_start) = result[start_idx..].find(&start_tag) {
            let block_start = start_idx + block_start;
            
            // Find the matching end tag
            if let Some(block_end) = result[block_start..].find(end_tag) {
                let block_end = block_start + block_end + end_tag.len();
                
                // If this is the selected value, keep the content but remove the tags
                if *value == selected_value {
                    let content_start = block_start + start_tag.len();
                    let content_end = block_end - end_tag.len();
                    
                    // Create a new string with the content but without the tags
                    let new_result = format!(
                        "{}{}{}",
                        &result[0..block_start],
                        &result[content_start..content_end],
                        &result[block_end..]
                    );
                    
                    result = new_result;
                    
                    // Adjust the start index for the next search
                    start_idx = block_start + (content_end - content_start);
                } else {
                    // This is not the selected value, remove the entire block
                    let new_result = format!(
                        "{}{}",
                        &result[0..block_start],
                        &result[block_end..]
                    );
                    
                    result = new_result;
                    
                    // Adjust the start index for the next search
                    start_idx = block_start;
                }
            } else {
                // No matching end tag found, break the loop
                break;
            }
        }
    }
    
    result
}
