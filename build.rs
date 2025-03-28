use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    // Tell Cargo to rerun this build script if any templates change
    println!("cargo:rerun-if-changed=templates/");
    
    // Copy the templates directory to the output directory
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_path = Path::new(&out_dir);
    
    // Include the config.json file in the build
    println!("cargo:rerun-if-changed=config.json");
    
    // Handle template files
    copy_templates_dir("templates", out_path);
    
    println!("cargo:rustc-env=FERRISUP_TEMPLATES_DIR={}", out_path.display());
}

fn copy_templates_dir<P: AsRef<Path>>(templates_dir: &str, out_dir: P) {
    let source_dir = PathBuf::from(templates_dir);
    let target_dir = out_dir.as_ref().join("templates");
    
    // Create the target directory if it doesn't exist
    fs::create_dir_all(&target_dir).expect("Failed to create templates directory");
    
    // Copy all template files
    copy_dir_recursively(&source_dir, &target_dir);
}

fn copy_dir_recursively(source: &Path, target: &Path) {
    // Create the target directory if it doesn't exist
    if !target.exists() {
        fs::create_dir_all(target).expect(&format!("Failed to create directory: {:?}", target));
    }
    
    // Iterate through the source directory and copy all files
    for entry in fs::read_dir(source).expect(&format!("Failed to read directory: {:?}", source)) {
        let entry = entry.expect("Failed to read directory entry");
        let entry_path = entry.path();
        let target_path = target.join(entry_path.file_name().unwrap());
        
        if entry_path.is_dir() {
            copy_dir_recursively(&entry_path, &target_path);
        } else {
            fs::copy(&entry_path, &target_path).expect(&format!(
                "Failed to copy {:?} to {:?}", entry_path, target_path
            ));
        }
    }
}
