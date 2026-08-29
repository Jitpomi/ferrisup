use anyhow::{Result, anyhow};
use colored::Colorize;
use dialoguer::Select;
use handlebars::Handlebars;
use lazy_static::lazy_static;
use serde_json::{Map, Value, json};
use std::fs;
use std::fs::File;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
// Cross-platform file permission handling
use ferrisup_common::to_pascal_case;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

lazy_static! {
    static ref CURRENT_VARIABLES: Arc<RwLock<Map<String, Value>>> =
        Arc::new(RwLock::new(Map::new()));
}

pub fn get_template(name: &str) -> Result<String> {
    let templates = get_all_templates()?;

    if templates.contains(&name.to_string()) {
        // Check if the template has a valid template.json file
        let template_dir = format!("{}/templates/{}", env!("CARGO_MANIFEST_DIR"), name);
        let template_json = Path::new(&template_dir).join("template.json");

        if template_json.exists() {
            Ok(name.to_string())
        } else {
            Err(anyhow!(
                "Template '{name}' has no template.json configuration"
            ))
        }
    } else {
        Err(anyhow!("Unknown template '{name}'"))
    }
}

pub fn get_all_templates() -> Result<Vec<String>> {
    // List all built-in templates
    let templates = vec![
        "minimal".to_string(),
        "library".to_string(),
        "embedded".to_string(),
        "server".to_string(),
        "client".to_string(),
        "serverless".to_string(),
        "data-science".to_string(),
        "edge".to_string(),
    ];

    Ok(templates)
}

/// Returns a list of templates with their descriptions
/// Format: Vec<(name, description)>
pub fn list_templates() -> Result<Vec<(String, String)>> {
    // Define core templates with descriptions
    // IMPORTANT: Only include the 8 core templates that are actually available in the new command
    let templates = vec![
        (
            "minimal".to_string(),
            "Simple binary with a single main.rs file".to_string(),
        ),
        (
            "library".to_string(),
            "Rust library crate with a lib.rs file".to_string(),
        ),
        (
            "embedded".to_string(),
            "Embedded systems firmware for microcontrollers".to_string(),
        ),
        (
            "server".to_string(),
            "Web server with API endpoints (Axum, Actix, or Poem)".to_string(),
        ),
        (
            "client".to_string(),
            "Frontend web application (Dioxus, Tauri, or Leptos)".to_string(),
        ),
        (
            "serverless".to_string(),
            "Serverless function (AWS Lambda, Cloudflare Workers, etc.)".to_string(),
        ),
        (
            "data-science".to_string(),
            "Data science and machine learning projects".to_string(),
        ),
        (
            "edge".to_string(),
            "Edge computing applications (Cloudflare, Vercel, Fastly, AWS, etc.)".to_string(),
        ),
    ];

    // Return only the core templates without discovering additional ones
    // This ensures the list matches exactly what's shown in the new command

    Ok(templates)
}

/// Get data science templates with descriptions
pub fn list_data_science_templates() -> Result<Vec<(String, String)>> {
    Ok(vec![
        // Data Analysis
        ("data-science/polars-cli".to_string(), "Data Analysis: Process and analyze data with Polars (similar to pandas)".to_string()),

        // Machine Learning
        ("data-science/linfa-examples".to_string(), "Machine Learning: Working examples with Linfa 0.7.1 (classification, regression, clustering)".to_string()),

    ])
}

/// Apply a template to a target directory
pub fn apply_template(
    template_name: &str,
    target_dir: &Path,
    project_name: &str,
    variables: Option<Value>,
) -> Result<()> {
    // Get the template configuration
    let template_config = get_template_config(template_name)?;

    // Check if the template has a redirect based on a variable
    if let Some(redirect) = template_config.get("redirect") {
        if let Some(redirect_obj) = redirect.as_object() {
            // If we have variables, check for redirects
            if let Some(vars) = variables.as_ref() {
                for (_key, value) in redirect_obj {
                    // For each redirect key, check if we have a matching variable
                    for (_var_name, var_value) in vars.as_object().unwrap_or(&Map::new()) {
                        if var_value.is_string() {
                            if let Some(redirect_path) = value.as_str().filter(|p| !p.is_empty()) {
                                // Apply the redirected template instead
                                return apply_template(
                                    redirect_path,
                                    target_dir,
                                    project_name,
                                    variables,
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    // Process the template files
    let template_dir = get_template_dir(template_name)?;

    // Register handlebars helpers
    let mut handlebars = Handlebars::new();
    handlebars.register_escape_fn(handlebars::no_escape);

    // Register the eq helper for conditional checks
    handlebars.register_helper(
        "eq",
        Box::new(
            |h: &handlebars::Helper<'_>,
             _: &handlebars::Handlebars<'_>,
             _: &handlebars::Context,
             _: &mut handlebars::RenderContext<'_, '_>,
             out: &mut dyn handlebars::Output|
             -> handlebars::HelperResult {
                let param1 = h.param(0).unwrap().value();
                let param2 = h.param(1).unwrap().value();
                out.write(&(param1 == param2).to_string())?;
                Ok(())
            },
        ),
    );

    // Prepare template variables
    let mut template_vars = json!({
        "project_name": project_name,
        "project_name_kebab": project_name.replace(" ", "-").to_lowercase(),
        "project_name_snake": project_name.replace(" ", "_").to_lowercase(),
        "project_name_pascal": to_pascal_case(project_name),
        "date": "2025-04-20",
        "year": "2025",
    });

    // Add user-provided variables
    if let Some(ref vars) = variables {
        if let Some(obj) = vars.as_object() {
            if let Some(obj_mut) = template_vars.as_object_mut() {
                for (_key, value) in obj {
                    obj_mut.insert(_key.clone(), value.clone());
                }
            }
        }
    }

    // Process template-specific options
    let options = template_config.get("options").and_then(|o| o.as_array());
    if let Some(options) = options {
        let vars = template_vars.as_object_mut().unwrap();

        // Only prompt for options if skip_framework_prompt is not set to true
        let skip_framework_prompt = variables
            .as_ref()
            .and_then(|v| v.get("skip_framework_prompt"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        if !skip_framework_prompt {
            for option in options {
                let option_obj = option.as_object().unwrap();
                let name = option_obj.get("name").unwrap().as_str().unwrap();
                let description = option_obj.get("description").unwrap().as_str().unwrap();

                // Skip server_framework prompt if we're applying a server/* template directly
                if name == "server_framework"
                    && (template_name == "server/axum"
                        || template_name == "server/actix"
                        || template_name == "server/poem")
                {
                    continue;
                }

                // Skip prompting if the value is already provided in variables
                if variables.as_ref().and_then(|v| v.get(name)).is_some() {
                    // Copy the value from input variables
                    if let Some(value) = variables.as_ref().and_then(|v| v.get(name)) {
                        vars.insert(name.to_string(), value.clone());
                    }
                    continue;
                }

                let option_type = option_obj.get("type").unwrap().as_str().unwrap();

                if option_type == "select" {
                    let options_array = option_obj.get("options").unwrap().as_array().unwrap();
                    let options: Vec<&str> =
                        options_array.iter().map(|o| o.as_str().unwrap()).collect();

                    let selection = Select::new()
                        .with_prompt(description)
                        .default(0)
                        .items(&options)
                        .interact()?;

                    let selected = options[selection];
                    println!("Using {} as the {}", selected, name);
                    vars.insert(name.to_string(), json!(selected));
                } else if option_type == "input" {
                    let default = option_obj
                        .get("default")
                        .map(|d| d.as_str().unwrap())
                        .unwrap_or("");
                    let value = prompt_with_default(description, default)?;
                    vars.insert(name.to_string(), json!(value));
                } else if option_type == "boolean" {
                    let default = option_obj
                        .get("default")
                        .map(|d| d.as_bool().unwrap())
                        .unwrap_or(false);
                    let options = if default {
                        &["yes", "no"]
                    } else {
                        &["no", "yes"]
                    };
                    let value = prompt_with_options(description, options)?;
                    let bool_value = value == "yes";
                    vars.insert(name.to_string(), json!(bool_value));
                }
            }
        }
    }

    // Process files specified in the template.json
    if let Some(files) = template_config.get("files").and_then(|f| f.as_array()) {
        for file in files {
            if let Some(file_obj) = file.as_object() {
                let source = file_obj.get("source").and_then(|s| s.as_str());
                let target = file_obj.get("target").and_then(|t| t.as_str());

                if let (Some(source_path), Some(target_path)) = (source, target) {
                    // Create parent directories for the target
                    let target_file = target_dir.join(target_path);
                    if let Some(parent) = target_file.parent() {
                        fs::create_dir_all(parent)?;
                    }

                    // Get content from template
                    let source_file = template_dir.join(source_path);

                    if !source_file.exists() {
                        return Err(anyhow!(
                            "Source file does not exist: {}",
                            source_file.display()
                        ));
                    }

                    let content = fs::read_to_string(&source_file).map_err(|e| {
                        anyhow!(
                            "Failed to read source file {}: {}",
                            source_file.display(),
                            e
                        )
                    })?;

                    // Apply template variables
                    let rendered = handlebars
                        .render_template(&content, &template_vars)
                        .map_err(|e| anyhow!("Failed to render template: {}", e))?;

                    // Write to target
                    fs::write(&target_file, rendered)?;

                    // If it's a script, make it executable on Unix
                    #[cfg(unix)]
                    {
                        // We already have the PermissionsExt import at the top level
                        let is_script = target_path.ends_with(".sh")
                            || content.starts_with("#!/bin/bash")
                            || content.starts_with("#!/usr/bin/env");

                        if is_script {
                            let metadata = fs::metadata(&target_file)?;
                            let mut perms = metadata.permissions();
                            #[cfg(unix)]
                            perms.set_mode(0o755); // rwxr-xr-x
                            fs::set_permissions(&target_file, perms)?;
                        }
                    }
                }
            }
        }
    }

    // Process dependencies if specified
    if let Some(dependencies) = template_config.get("dependencies") {
        process_dependencies(dependencies, target_dir, "dependencies")?;
    }

    // For data science templates, we only want to copy the files explicitly listed in the files section
    // to avoid copying files from other data formats (CSV, JSON, Parquet)
    let skip_dir_copying = template_name.starts_with("data-science/");

    // Only copy directory contents for non-data-science templates
    if !skip_dir_copying {
        // Copy remaining files from the template directory
        // We need to handle template variables in all files
        let mut handlebars = Handlebars::new();
        handlebars.register_escape_fn(handlebars::no_escape);

        // Add template dir to variable set for use in templates
        if let Some(template_vars_obj) = template_vars.as_object_mut() {
            template_vars_obj.insert(
                "template_dir".to_string(),
                json!(template_dir.to_string_lossy().to_string()),
            );
        }

        // Process template files with variables
        process_template_directory(&template_dir, &target_dir, &template_vars, &mut handlebars)?;
    }

    // Print successful message
    println!(
        "\n✅ {} project created successfully!",
        project_name.green()
    );

    if let Some(next_steps) =
        get_template_next_steps(template_name, project_name, Some(template_vars.clone()))
    {
        println!("\n{}", "Next steps:".bold().green());
        for step in next_steps {
            println!("- {}", step);
        }
    }

    Ok(())
}

/// Process template files with variable substitution in a directory
fn process_template_directory(
    src: &Path,
    dst: &Path,
    template_vars: &Value,
    handlebars: &mut Handlebars,
) -> Result<()> {
    fs::create_dir_all(dst)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            let file_name = entry.file_name();
            let file_name_str = file_name.to_string_lossy();

            // Determine target path - remove .template extension if present
            let target_file_name = if file_name_str.ends_with(".template") {
                file_name_str.replace(".template", "")
            } else {
                file_name_str.to_string()
            };

            let target_path = dst.join(&target_file_name);

            // Process files that need template variable substitution
            if path.extension().map_or(false, |ext| {
                ext == "template"
                    || ext == "rs"
                    || ext == "md"
                    || ext == "toml"
                    || ext == "html"
                    || ext == "css"
                    || ext == "json"
                    || ext == "yml"
                    || ext == "yaml"
            }) {
                // Read template content
                let template_content = fs::read_to_string(&path)?;

                // Process conditional blocks
                let processed_content =
                    process_conditional_blocks(&template_content, template_vars)?;

                // Render with handlebars
                let rendered = handlebars
                    .render_template(&processed_content, template_vars)
                    .map_err(|e| anyhow!("Failed to render template: {}", e))?;

                // Write rendered content
                let mut file = File::create(&target_path)?;
                file.write_all(rendered.as_bytes())?;
            } else {
                // Just copy other files without processing
                fs::copy(&path, &target_path)?;
            }

            // Set executable bit for .sh files
            if target_path.extension().map_or(false, |ext| ext == "sh") {
                // Set executable permissions in a cross-platform way
                #[cfg(unix)]
                {
                    let mut perms = fs::metadata(&target_path)?.permissions();
                    perms.set_mode(perms.mode() | 0o111); // Add execute bit
                    fs::set_permissions(&target_path, perms)?;
                }
                // On Windows, we don't need to set execute permissions explicitly
                #[cfg(not(unix))]
                {
                    // Windows doesn't have the concept of executable bit
                    // The OS determines if a file is executable based on its extension
                }
            }
        } else if path.is_dir() {
            // Skip .git directory, .github directory, etc.
            if entry.file_name() == ".git"
                || entry.file_name() == ".github"
                || entry.file_name() == "target"
                || entry.file_name() == "node_modules"
            {
                continue;
            }

            // Process subdirectory recursively
            process_template_directory(
                &path,
                &dst.join(entry.file_name()),
                template_vars,
                handlebars,
            )?;
        }
    }

    Ok(())
}

#[allow(dead_code)]
fn process_file(
    file_entry: &Value,
    template_dir: &Path,
    target_dir: &Path,
    template_vars: &Value,
    handlebars: &mut Handlebars,
) -> Result<()> {
    if let (Some(source), Some(target)) = (
        file_entry.get("source").and_then(|s| s.as_str()),
        file_entry.get("target").and_then(|t| t.as_str()),
    ) {
        // Check if there's a condition and evaluate it
        if let Some(condition) = file_entry.get("condition").and_then(|c| c.as_str()) {
            // Parse and evaluate the condition
            let vars = template_vars.as_object().unwrap();

            // Simple condition evaluation for now - just check equality
            // Format: "variable_name == 'value'"
            let parts: Vec<&str> = condition.split("==").collect();
            if parts.len() == 2 {
                let var_name = parts[0].trim();
                let expected_value = parts[1].trim().trim_matches('\'').trim_matches('"');

                if let Some(_var_value) = vars.get(var_name) {
                    if let Some(value_str) = _var_value.as_str() {
                        if value_str != expected_value {
                            // Condition not met, skip this file
                            return Ok(());
                        }
                    }
                } else {
                    // Variable not found, skip this file
                    return Ok(());
                }
            }
        }

        let source_path = template_dir.join(source);
        let mut target_path = target_dir.join(target);

        // Remove .template extension from the target path if present
        if let Some(filename) = target_path.file_name() {
            let filename_str = filename.to_string_lossy();
            if filename_str.ends_with(".template") {
                let new_filename = filename_str.replace(".template", "");
                target_path.pop(); // Remove the old filename
                target_path.push(new_filename); // Add the new filename without .template
            }
        }

        // Create parent directories if they don't exist
        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Process the file based on its extension
        if source.ends_with(".template")
            || source.ends_with(".rs")
            || source.ends_with(".md")
            || source.ends_with(".toml")
            || source.ends_with(".html")
            || source.ends_with(".css")
            || source.ends_with(".json")
            || target.ends_with("Cargo.toml")
        {
            // Read the template content
            let template_content = fs::read_to_string(&source_path)?;

            // Process conditional blocks manually before rendering with Handlebars
            let processed_content = process_conditional_blocks(&template_content, template_vars)?;

            // Render the template with variables using Handlebars
            let rendered = handlebars.render_template(&processed_content, template_vars)?;

            // Write the rendered content to the target file
            let mut file = File::create(&target_path)?;
            file.write_all(rendered.as_bytes())?;
        } else {
            // Just copy the file
            fs::copy(&source_path, &target_path)?;
            // Set executable bit for .sh files
            if let Some(ext) = target_path.extension() {
                if ext == "sh" {
                    #[cfg(unix)]
                    {
                        let mut perms = fs::metadata(&target_path)?.permissions();
                        perms.set_mode(perms.mode() | 0o111); // Add execute bit
                        fs::set_permissions(&target_path, perms)?;
                    }
                    #[cfg(not(unix))]
                    {
                        // Windows doesn't have the concept of executable bit
                        // The OS determines if a file is executable based on its extension
                    }
                }
            }
        }
    }

    Ok(())
}

#[allow(dead_code)]
fn process_conditional_blocks(content: &str, variables: &Value) -> Result<String> {
    let mut result = content.to_string();

    // Get the cloud provider from variables
    let cloud_provider =
        if let Some(provider) = variables.get("cloud_provider").and_then(|p| p.as_str()) {
            provider
        } else {
            return Ok(result);
        };

    // Process {{#if (eq cloud_provider "aws")}} blocks
    let providers = ["aws", "gcp", "azure", "vercel", "netlify"];

    for provider in providers {
        let start_tag = format!("{{{{#if (eq cloud_provider \"{}\")}}}}", provider);
        let end_tag = "{{/if}}";

        // Find all blocks for this provider
        let mut start_idx = 0;
        while let Some(block_start) = result[start_idx..].find(&start_tag) {
            let block_start = start_idx + block_start;

            // Find the matching end tag
            if let Some(block_end) = result[block_start..].find(end_tag) {
                let block_end = block_start + block_end + end_tag.len();

                // If this is the selected provider, keep the content but remove the tags
                if provider == cloud_provider {
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
                    // This is not the selected provider, remove the entire block
                    let new_result = format!("{}{}", &result[0..block_start], &result[block_end..]);

                    result = new_result;

                    // Adjust the start index for the next search
                    start_idx = block_start;
                }
            } else {
                // No matching end tag found, move past this start tag
                start_idx = block_start + start_tag.len();
            }
        }
    }

    Ok(result)
}

#[allow(dead_code)]
fn apply_transformations(
    content: &str,
    transformations: &[Value],
    variables: &Value,
) -> Result<String> {
    let mut result = content.to_string();

    for transformation in transformations {
        if let Some(pattern) = transformation.get("pattern").and_then(|p| p.as_str()) {
            if let Some(replacement_value) = transformation.get("replacement") {
                // If replacement is an object, it may contain variable references
                if let Some(replacement_obj) = replacement_value.as_object() {
                    // Check for variable matches in the replacement object
                    if let Some(vars) = variables.as_object() {
                        for (_var_name, _var_value) in vars {
                            if let Some(replacement) = replacement_obj.get(_var_name) {
                                if let Some(replacement_str) = replacement.as_str() {
                                    result = result.replace(pattern, replacement_str);
                                }
                            }
                        }
                    }
                } else if let Some(replacement_str) = replacement_value.as_str() {
                    // Direct string replacement
                    result = result.replace(pattern, replacement_str);
                }
            }
        }
    }

    Ok(result)
}

/// Process dependencies from template.json
fn process_dependencies(dependencies: &Value, _target_dir: &Path, section: &str) -> Result<()> {
    if let Some(deps) = dependencies.as_object() {
        for (_key, value) in deps {
            if let Some(dep_name) = value.get("name").and_then(|n| n.as_str()) {
                let mut version = "latest".to_string();
                if let Some(ver) = value.get("version").and_then(|v| v.as_str()) {
                    version = ver.to_string();
                }

                println!(
                    "📦 Adding {} dependency: {} ({})",
                    section, dep_name, version
                );
            }
        }
    }

    Ok(())
}

#[allow(dead_code)]
fn get_template_dir(template_name: &str) -> Result<PathBuf> {
    let templates_dir = format!("{}/templates", env!("CARGO_MANIFEST_DIR"));

    // Check if it's a direct template
    let direct_path = Path::new(&templates_dir).join(template_name);
    if direct_path.exists() && direct_path.is_dir() {
        return Ok(direct_path);
    }

    // Check if it's a nested template (e.g., client/leptos/counter)
    let parts: Vec<&str> = template_name.split('/').collect();
    if parts.len() > 1 {
        let nested_path = Path::new(&templates_dir).join(parts.join("/"));
        if nested_path.exists() && nested_path.is_dir() {
            return Ok(nested_path);
        }
    }

    // Search for the template in subdirectories
    for entry in fs::read_dir(&templates_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            // Check if this directory contains our template
            let potential_template = path.join(template_name);
            if potential_template.exists() && potential_template.is_dir() {
                return Ok(potential_template);
            }

            // Check one level deeper
            if let Ok(subentries) = fs::read_dir(&path) {
                for subentry in subentries {
                    let subentry = subentry?;
                    let subpath = subentry.path();

                    if subpath.is_dir() {
                        let subdir_name = subpath.file_name().unwrap().to_string_lossy();
                        if subdir_name == template_name {
                            return Ok(subpath);
                        }
                    }
                }
            }
        }
    }

    Err(anyhow::anyhow!("Template '{}' not found", template_name))
}

/// Get the template configuration
pub fn get_template_config(template_name: &str) -> Result<Value> {
    let template_dir = get_template_dir(template_name)?;

    // Read the template configuration
    let template_config_path = template_dir.join("template.json");
    let template_config_str = fs::read_to_string(&template_config_path)?;
    let template_config: Value = serde_json::from_str(&template_config_str)?;

    Ok(template_config)
}

#[allow(dead_code)]
fn replace_variables(content: &str, variables: &Value) -> String {
    let mut result = content.to_string();

    if let Some(obj) = variables.as_object() {
        for (_key, value) in obj {
            let placeholder = format!("{{{{{}}}}}", _key);
            let replacement = match value {
                Value::String(s) => s.clone(),
                _ => value.to_string(),
            };

            result = result.replace(&placeholder, &replacement);
        }
    }

    result
}

/// Find the directory containing a template
pub fn find_template_directory(template_name: &str) -> Result<PathBuf> {
    let templates_dir = format!("{}/templates", env!("CARGO_MANIFEST_DIR"));

    // Check if it's a direct template
    let direct_path = Path::new(&templates_dir).join(template_name);
    if direct_path.exists() && direct_path.is_dir() {
        return Ok(direct_path);
    }

    // Check if it's a nested template (e.g., client/leptos/counter)
    let parts: Vec<&str> = template_name.split('/').collect();
    if parts.len() > 1 {
        let nested_path = Path::new(&templates_dir).join(parts.join("/"));
        if nested_path.exists() && nested_path.is_dir() {
            return Ok(nested_path);
        }
    }

    // Search for the template in subdirectories
    for entry in fs::read_dir(&templates_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            // Check if this directory contains our template
            let potential_template = path.join(template_name);
            if potential_template.exists() && potential_template.is_dir() {
                return Ok(potential_template);
            }

            // Check one level deeper
            if let Ok(subentries) = fs::read_dir(&path) {
                for subentry in subentries {
                    let subentry = subentry?;
                    let subpath = subentry.path();

                    if subpath.is_dir() {
                        let subdir_name = subpath.file_name().unwrap().to_string_lossy();
                        if subdir_name == template_name {
                            return Ok(subpath);
                        }
                    }
                }
            }
        }
    }

    Err(anyhow::anyhow!("Template '{}' not found", template_name))
}

/// Get the template's custom next steps if available
pub fn get_template_next_steps(
    template_name: &str,
    project_name: &str,
    variables: Option<Value>,
) -> Option<Vec<String>> {
    // Check if there's a .ferrisup_next_steps.json file in the project directory
    let project_dir = Path::new(project_name);
    let next_steps_file = project_dir.join(".ferrisup_next_steps.json");

    if next_steps_file.exists() {
        // Load and parse the next steps from the JSON file
        match fs::read_to_string(&next_steps_file) {
            Ok(content) => {
                match serde_json::from_str::<Value>(&content) {
                    Ok(json) => {
                        if let Some(steps) = json.get("next_steps").and_then(|s| s.as_array()) {
                            let next_steps: Vec<String> = steps
                                .iter()
                                .filter_map(|s| s.as_str().map(|s| s.to_string()))
                                .collect();

                            // Delete the file after reading
                            let _ = fs::remove_file(&next_steps_file);

                            if !next_steps.is_empty() {
                                return Some(next_steps);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to parse next steps JSON: {}", e);
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to read next steps file: {}", e);
            }
        }
    }

    // If no next_steps.json file or it's invalid, try to get from template.json
    if let Ok(template_config) = get_template_config(template_name) {
        if let Some(next_steps) = template_config.get("next_steps") {
            // Check if next_steps is an array
            if let Some(steps) = next_steps.as_array() {
                let mut result = Vec::new();

                for step in steps {
                    if let Some(step_str) = step.as_str() {
                        let mut step_text = step_str.to_string();

                        // Replace variables in the step text
                        if let Some(vars) = &variables {
                            // Create a Handlebars instance for rendering
                            let mut handlebars = Handlebars::new();
                            handlebars.register_escape_fn(handlebars::no_escape);

                            // Replace variables in the step text
                            if let Ok(rendered) = handlebars.render_template(&step_text, vars) {
                                step_text = rendered;
                            }
                        }

                        // Replace {{project_name}} with the actual project name
                        step_text = step_text.replace("{{project_name}}", project_name);

                        result.push(step_text);
                    }
                }

                if !result.is_empty() {
                    return Some(result);
                }
            }

            // Check if next_steps is an object with conditional steps
            if let Some(steps_obj) = next_steps.as_object() {
                // If we have variables, try to find the matching steps
                if let Some(vars) = &variables {
                    // Try to match based on data_format variable
                    if let Some(data_format) = vars.get("data_format").and_then(|f| f.as_str()) {
                        if let Some(format_steps) =
                            steps_obj.get(data_format).and_then(|s| s.as_array())
                        {
                            let mut result = Vec::new();

                            for step in format_steps {
                                if let Some(step_str) = step.as_str() {
                                    // Replace {{project_name}} with the actual project name
                                    let step_text =
                                        step_str.replace("{{project_name}}", project_name);
                                    result.push(step_text);
                                }
                            }

                            if !result.is_empty() {
                                return Some(result);
                            }
                        }
                    }

                    // Try to match based on platform variable
                    if let Some(platform) = vars.get("platform").and_then(|p| p.as_str()) {
                        if let Some(platform_steps) =
                            steps_obj.get(platform).and_then(|s| s.as_array())
                        {
                            let mut result = Vec::new();

                            for step in platform_steps {
                                if let Some(step_str) = step.as_str() {
                                    // Replace {{project_name}} with the actual project name
                                    let step_text =
                                        step_str.replace("{{project_name}}", project_name);
                                    result.push(step_text);
                                }
                            }

                            if !result.is_empty() {
                                return Some(result);
                            }
                        }
                    }

                    // Try to match based on other variables
                    for (_var_name, var_value) in vars.as_object().unwrap() {
                        if let Some(var_str) = var_value.as_str() {
                            if let Some(var_steps) =
                                steps_obj.get(var_str).and_then(|s| s.as_array())
                            {
                                let mut result = Vec::new();

                                for step in var_steps {
                                    if let Some(step_str) = step.as_str() {
                                        // Replace {{project_name}} with the actual project name
                                        let step_text =
                                            step_str.replace("{{project_name}}", project_name);
                                        result.push(step_text);
                                    }
                                }

                                if !result.is_empty() {
                                    return Some(result);
                                }
                            }
                        }
                    }
                }

                // If no specific match found, try to use default steps
                if let Some(default_steps) = steps_obj.get("default").and_then(|s| s.as_array()) {
                    let mut result = Vec::new();

                    for step in default_steps {
                        if let Some(step_str) = step.as_str() {
                            // Replace {{project_name}} with the actual project name
                            let step_text = step_str.replace("{{project_name}}", project_name);
                            result.push(step_text);
                        }
                    }

                    if !result.is_empty() {
                        return Some(result);
                    }
                }
            }
        }
    }

    // Default steps if no specific steps found
    Some(vec![
        format!("🚀 Navigate to your project: cd {}", project_name),
        "📝 Review the generated code".to_string(),
        "🔧 Build the project: cargo build".to_string(),
        "▶️ Run the project: cargo run".to_string(),
    ])
}

/// Prompt the user with a question and return their answer
#[allow(dead_code)]
fn prompt(question: &str) -> Result<String> {
    print!("{} ", question);
    io::stdout().flush()?;

    let stdin = io::stdin();
    let mut line = String::new();
    stdin.lock().read_line(&mut line)?;

    Ok(line.trim().to_string())
}

/// Prompt the user with a question and a set of options
fn prompt_with_options(question: &str, options: &[&str]) -> Result<String> {
    let selection = Select::new()
        .with_prompt(question)
        .default(0)
        .items(options)
        .interact()?;

    Ok(options[selection].to_string())
}

/// Prompt the user with a question and a default value
fn prompt_with_default(question: &str, default: &str) -> Result<String> {
    print!("{} [{}]: ", question, default);
    io::stdout().flush()?;

    let stdin = io::stdin();
    let mut line = String::new();
    stdin.lock().read_line(&mut line)?;

    let input = line.trim();
    if input.is_empty() {
        Ok(default.to_string())
    } else {
        Ok(input.to_string())
    }
}
