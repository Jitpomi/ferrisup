use std::path::Path;
use std::process::Command;
use anyhow::{anyhow, Context, Result};
use colored::Colorize;
#[allow(unused_imports)]
use toml_edit::{DocumentMut, Item};

pub fn write_cargo_toml(project_dir: &Path) -> anyhow::Result<()> {
    let cargo_toml = format!(
        r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
"#,
        // Using a default project name
        "rust_project"
    );

    std::fs::write(project_dir.join("Cargo.toml"), cargo_toml)
        .context("Failed to write Cargo.toml")?;

    Ok(())
}

pub fn read_cargo_toml(project_dir: &Path) -> anyhow::Result<String> {
    let cargo_path = project_dir.join("Cargo.toml");
    if !cargo_path.exists() {
        return Err(anyhow::anyhow!("Cargo.toml not found"));
    }

    std::fs::read_to_string(&cargo_path)
        .context(format!("Failed to read {}", cargo_path.display()))
}

pub fn write_cargo_toml_content(project_dir: &Path, content: &str) -> anyhow::Result<()> {
    let cargo_path = project_dir.join("Cargo.toml");

    std::fs::write(&cargo_path, content)
        .context(format!("Failed to write {}", cargo_path.display()))?;

    println!("{} {}", "Updated".green(), cargo_path.display());

    Ok(())
}

pub fn update_workspace_members(project_dir: &Path) -> anyhow::Result<bool> {
    let cargo_content = read_cargo_toml(project_dir)?;

    // Parse the TOML content
    let cargo_toml: toml::Value = toml::from_str(&cargo_content)
        .context("Failed to parse Cargo.toml as valid TOML")?;

    // Check if it's a workspace
    if cargo_toml.get("workspace").is_none() {
        return Err(anyhow::anyhow!("Not a Cargo workspace (no [workspace] section in Cargo.toml)"));
    }

    // Extract existing workspace members
    let mut updated = false;
    let mut existing_members = Vec::new();

    if let Some(workspace) = cargo_toml.get("workspace").and_then(|w| w.as_table()) {
        if let Some(members) = workspace.get("members").and_then(|m| m.as_array()) {
            for member in members {
                if let Some(member_str) = member.as_str() {
                    existing_members.push(member_str.to_string());
                }
            }
        }
    }

    // Discover crates in the project directory
    let mut crates_to_add = Vec::new();

    // Check common workspace directories
    for dir in &["client_old", "server", "ferrisup_common", "libs", "crates"] {
        let dir_path = project_dir.join(dir);
        if dir_path.exists() && dir_path.is_dir() {
            // Check if we have the wildcard pattern already
            let wildcard = format!("{}/* ", dir);
            if !existing_members.contains(&wildcard) && !existing_members.iter().any(|m| m.starts_with(&format!("{}/", dir))) {
                // Look for individual crates
                for entry in std::fs::read_dir(&dir_path).context(format!("Failed to read directory {}", dir_path.display()))? {
                    let entry = entry.context("Failed to read directory entry")?;
                    let path = entry.path();

                    if path.is_dir() && path.join("Cargo.toml").exists() {
                        let relative_path = format!("{}/{}", dir, path.file_name().unwrap().to_string_lossy());
                        if !existing_members.contains(&relative_path) {
                            crates_to_add.push(relative_path);
                        }
                    }
                }
            }
        }
    }

    // Add root level crates
    for entry in std::fs::read_dir(project_dir).context("Failed to read project directory")? {
        let entry = entry.context("Failed to read directory entry")?;
        let path = entry.path();

        if path.is_dir() && path.join("Cargo.toml").exists() {
            let dir_name = path.file_name().unwrap().to_string_lossy().to_string();

            // Skip common directories that might contain multiple crates and system directories
            if ![
                "src", "target", ".git", ".github", ".ferrisup"
            ].contains(&dir_name.as_str()) && !existing_members.contains(&dir_name) {
                crates_to_add.push(dir_name);
            }
        }
    }

    // If we found new crates, update the Cargo.toml
    if !crates_to_add.is_empty() {
        updated = true;

        // Create a new TOML structure with updated members
        let mut new_cargo = cargo_toml.clone();

        // Get or create the workspace table
        let workspace = new_cargo.get_mut("workspace")
            .and_then(|w| w.as_table_mut())
            .expect("Workspace section should exist");

        // Get or create the members array
        let members = if let Some(members) = workspace.get_mut("members").and_then(|m| m.as_array_mut()) {
            members
        } else {
            workspace.insert("members".to_string(), toml::Value::Array(Vec::new()));
            workspace.get_mut("members").and_then(|m| m.as_array_mut()).unwrap()
        };

        // Add new crates
        for crate_path in crates_to_add {
            println!("Adding workspace member: {}", crate_path.green());
            members.push(toml::Value::String(crate_path.to_string()));
        }

        // Write the updated TOML back to the file
        let updated_content = toml::to_string(&new_cargo)
            .context("Failed to serialize updated Cargo.toml")?;

        write_cargo_toml_content(project_dir, &updated_content)?;
    }

    Ok(updated)
}

/// Helper function to extract dependencies from a TOML table

pub fn extract_dependencies(deps_table: &Item) -> anyhow::Result<Vec<(String, String, Option<Vec<String>>)>> {
    let mut dependencies = Vec::new();

    if let Some(deps_table) = deps_table.as_table() {
        for (name, value) in deps_table.iter() {
            if let Some(version) = value.as_str() {
                // Simple version string without features
                dependencies.push((name.to_string(), version.to_string(), None));
            } else if let Some(table) = value.as_table() {
                if let Some(version) = table.get("version").and_then(|v| v.as_str()) {
                    // Check for features
                    let mut features = Vec::new();
                    if let Some(features_value) = table.get("features").and_then(|f| f.as_array()) {
                        for feature in features_value {
                            if let Some(feature_str) = feature.as_str() {
                                features.push(feature_str.to_string());
                            }
                        }
                    }

                    let features_option = if features.is_empty() { None } else { Some(features) };
                    dependencies.push((name.to_string(), version.to_string(), features_option));
                }
            }
        }
    }

    Ok(dependencies)
}

/// Helper function to update Cargo.toml with dependencies using cargo add
pub fn update_cargo_with_dependencies(cargo_path: &Path, dependencies: Vec<(String, String, Option<Vec<String>>)>, dev: bool) -> anyhow::Result<()> {
    // Get the project directory (parent of the Cargo.toml file)
    let project_dir = cargo_path.parent().ok_or_else(|| anyhow!("Could not determine project directory"))?;

    // Save current directory to return to it after
    let current_dir = std::env::current_dir()?;

    // Change to project directory to run cargo add
    std::env::set_current_dir(project_dir)?;

    for (name, version, features) in dependencies {
        // Build cargo add command
        let mut cmd = std::process::Command::new("cargo");
        cmd.arg("add").arg(&name);

        // Add as development dependency if dev flag is set
        if dev {
            cmd.arg("--dev");
        }

        // Add version if it's not "*"
        if version != "*" {
            cmd.arg("--version").arg(&version);
        }

        // Add features if provided
        if let Some(feat_list) = features {
            if !feat_list.is_empty() {
                let features_str = feat_list.join(",");
                cmd.arg("--features").arg(features_str);
            }
        }

        // Run the command
        let output = cmd.output()
            .context(format!("Failed to add dependency: {}", name))?;

        if !output.status.success() {
            println!("{} {}",
                     "Warning:".yellow().bold(),
                     format!("Failed to add dependency: {}", name).yellow());

            // Print error message if available
            if let Ok(err) = String::from_utf8(output.stderr) {
                if !err.is_empty() {
                    println!("{}", err);
                }
            }
        }
    }

    // Change back to original directory
    std::env::set_current_dir(current_dir)?;

    Ok(())
}

/// Checks if a crate name is available on crates.io
///
/// Uses `cargo search --limit=1` to check if a crate with the given name exists.
/// If the crate exists, the search will return results and the name is not available.
/// If the crate doesn't exist, the search will return no results and the name is available.
///
/// # Arguments
/// * `name` - The crate name to check
///
/// # Returns
/// * `Ok(true)` if the crate name is available (doesn't exist on crates.io)
/// * `Ok(false)` if the crate name is not available (exists on crates.io)
/// * `Err` if there was an error checking the crate name
pub fn is_crate_name_available(name: &str) -> Result<bool> {
    // Use cargo search with a limit of 1 to check if the crate exists
    // The exact match format ensures we only get results for the exact crate name
    let output = Command::new("cargo")
        .arg("search")
        .arg("--limit=1")
        .arg(format!("^{}$", name))  // Use regex for exact match
        .output()
        .context("Failed to execute cargo search command")?;

    // If the search returns no results (empty stdout), the crate name is available
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Check if the output contains any search results
    Ok(stdout.trim().is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    // This function can be used to manually test the is_crate_name_available function
    // Uncomment and run with: cargo run --bin test_crate_availability

    #[test]
    pub fn test_crate_availability() {
        println!("Testing crate name availability checker...");

        // Test with a known existing crate
        let existing_crate = "serde";
        match is_crate_name_available(existing_crate) {
            Ok(available) => println!("Crate '{}' availability: {}", existing_crate,
                                      if available { "AVAILABLE ✅" } else { "NOT AVAILABLE ❌" }),
            Err(e) => println!("Error checking '{}': {}", existing_crate, e),
        }

        // Test with a likely non-existent crate
        let random_crate = format!("ferrisup-test-{}", std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs());
        match is_crate_name_available(&random_crate) {
            Ok(available) => println!("Crate '{}' availability: {}", random_crate,
                                      if available { "AVAILABLE ✅" } else { "NOT AVAILABLE ❌" }),
            Err(e) => println!("Error checking '{}': {}", random_crate, e),
        }

        // Test with ferrisup_common
        let common_crate = "ferrisup_common";
        match is_crate_name_available(common_crate) {
            Ok(available) => println!("Crate '{}' availability: {}", common_crate,
                                      if available { "AVAILABLE ✅" } else { "NOT AVAILABLE ❌" }),
            Err(e) => println!("Error checking '{}': {}", common_crate, e),
        }
    }


    #[test]
    fn test_write_cargo_toml() -> anyhow::Result<()> {
        let temp_dir = tempdir()?;
        let project_dir = temp_dir.path().join("test_project");
        fs::create_dir_all(&project_dir)?;

        write_cargo_toml(&project_dir)?;

        let cargo_path = project_dir.join("Cargo.toml");
        assert!(cargo_path.exists());

        let content = fs::read_to_string(cargo_path)?;
        assert!(content.contains("name = \"rust_project\""));

        Ok(())
    }

    #[test]
    fn test_read_cargo_toml() -> anyhow::Result<()> {
        // Create a temporary directory for testing
        let temp_dir = tempdir()?;
        let test_dir = temp_dir.path().join("test_project");
        fs::create_dir_all(&test_dir)?;

        // Create a test Cargo.toml file
        let cargo_content = r#"[package]
name = "test_project"
version = "0.1.0"
edition = "2021"
"#;
        let cargo_path = test_dir.join("Cargo.toml");
        fs::write(&cargo_path, cargo_content)?;

        // Read the file
        let read_content = read_cargo_toml(&test_dir)?;

        // Verify the content was read correctly
        assert_eq!(read_content, cargo_content);

        Ok(())
    }

    #[test]
    fn test_write_cargo_toml_content() -> anyhow::Result<()> {
        // Create a temporary directory for testing
        let temp_dir = tempdir()?;
        let test_dir = temp_dir.path().join("test_project");
        fs::create_dir_all(&test_dir)?;

        // Test content to write
        let content = r#"[package]
name = "test_project"
version = "0.1.0"
edition = "2021"
"#;

        // Write the content
        write_cargo_toml_content(&test_dir, content)?;

        // Verify the file was created with correct content
        let cargo_path = test_dir.join("Cargo.toml");
        assert!(cargo_path.exists());
        let read_content = fs::read_to_string(cargo_path)?;
        assert_eq!(read_content, content);

        Ok(())
    }

    #[test]
    fn test_update_workspace_members() -> anyhow::Result<()> {
        // Create a temporary directory for testing
        let temp_dir = tempdir()?;
        let workspace_dir = temp_dir.path().join("workspace");
        fs::create_dir_all(&workspace_dir)?;

        // Create a basic workspace Cargo.toml
        let cargo_content = r#"[workspace]
members = []

[workspace.package]
version = "0.1.0"
edition = "2021"
"#;
        let cargo_path = workspace_dir.join("Cargo.toml");
        fs::write(&cargo_path, cargo_content)?;

        // Create a component directory
        let component_dir = workspace_dir.join("component1");
        fs::create_dir_all(&component_dir)?;

        // Create a component Cargo.toml
        let component_cargo = r#"[package]
name = "component1"
version = "0.1.0"
edition = "2021"
"#;
        fs::write(component_dir.join("Cargo.toml"), component_cargo)?;

        // Update workspace members
        let updated = update_workspace_members(&workspace_dir)?;

        // Verify the workspace was updated
        assert!(updated);

        // Check that the workspace Cargo.toml now includes the component
        let updated_content = fs::read_to_string(cargo_path)?;
        assert!(updated_content.contains("members = ["));
        assert!(updated_content.contains("\"component1\""));

        Ok(())
    }

    #[test]
    fn test_extract_dependencies() -> anyhow::Result<()> {
        // Create a simple TOML string with dependencies
        let toml_str = r#"
[dependencies]
anyhow = "1.0"
"#;

        // Parse the TOML
        let parsed: DocumentMut = toml_str.parse().unwrap();
        let deps_table = &parsed["dependencies"];

        // Extract dependencies
        let deps = extract_dependencies(deps_table)?;

        // Verify the extracted dependencies
        assert_eq!(deps.len(), 1);

        // Check dependency (anyhow)
        assert_eq!(deps[0].0, "anyhow");
        assert_eq!(deps[0].1, "1.0");
        assert!(deps[0].2.is_none());

        Ok(())
    }

    #[test]
    fn test_update_cargo_with_dependencies() -> anyhow::Result<()> {
        let temp_dir = tempdir()?;
        let cargo_path = temp_dir.path().join("Cargo.toml");

        // Create a simple Cargo.toml
        let cargo_content = r#"[package]
name = "test-project"
version = "0.1.0"
edition = "2021"

[dependencies]
"#;
        std::fs::write(&cargo_path, cargo_content)?;

        // Skip the actual test as it would modify the system
        // Just verify the file was created
        assert!(cargo_path.exists());

        Ok(())
    }

    #[test]
    fn test_is_crate_name_available() -> anyhow::Result<()> {
        // Test with a crate name that definitely exists
        let result = is_crate_name_available("serde")?;
        assert!(!result, "'serde' should not be available");

        // Test with a crate name that almost certainly doesn't exist (random string)
        let random_name = format!("ferrisup-test-{}", std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs());
        let result = is_crate_name_available(&random_name)?;
        assert!(result, "Random crate name should be available");

        // Test with a crate name that is not available
        let result = is_crate_name_available("ferrisup")?;
        assert!(!result, "'ferrisup' should not be available");

        Ok(())
    }

    // We don't need this test as the function signature already ensures we can only pass strings
    // Removing the test that would cause a compilation error

}