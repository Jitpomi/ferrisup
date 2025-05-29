use std::path::Path;

// Function to fix imports in a newly created component
fn fix_component_imports(component_dir: &Path, component_name: &str, project_name: &str) -> Result<()> {
    println!("{}", format!("Fixing imports in component: {}", component_name).blue());
    
    // Get all Rust files in the component directory
    let mut rust_files = Vec::new();
    let src_dir = component_dir.join("src");
    
    if src_dir.exists() {
        for entry in walkdir::WalkDir::new(&src_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file()) {
                
            let path = entry.path();
            if let Some(ext) = path.extension() {
                if ext == "rs" {
                    rust_files.push(path.to_path_buf());
                }
            }
        }
    }
    
    // Fix imports in each Rust file
    for file_path in rust_files {
        // Read file content
        let content = fs::read_to_string(&file_path)?;
        
        // Replace imports like "use client::*;" with "use app_client::*;"
        let re = regex::Regex::new(&format!(r"use\s+{}(::|\s+)", component_name))
            .context("Failed to create regex")?;
            
        let new_package_name = format!("{}_{}", project_name, component_name);
        let updated_content = re.replace_all(&content, format!("use {} ", new_package_name));
        
        // Write updated content back to file
        fs::write(&file_path, updated_content.as_bytes())?;
        println!("  Fixed imports in: {}", file_path.display());
    }
    
    Ok(())
}
