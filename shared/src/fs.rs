use std::fs;
use std::path::Path;

#[allow(dead_code)]
pub fn create_directory(path: &Path) -> anyhow::Result<()> {
    if !path.exists() {
        fs::create_dir_all(path)?;
        println!("Created directory: {}", path.display());
    }
    Ok(())
}

#[allow(dead_code)]
pub fn copy_directory(src: &Path, dst: &Path) -> anyhow::Result<()> {
    // Create the destination directory if it doesn't exist
    create_directory(dst)?;
    
    // Use walkdir for robust directory traversal
    for entry in walkdir::WalkDir::new(src).follow_links(true).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        // Skip the root directory itself
        if path == src {
            continue;
        }
        
        // Get the path relative to the source directory
        let relative = path.strip_prefix(src)?;
        let target = dst.join(relative);
        
        if path.is_file() {
            // Create parent directories if needed
            if let Some(parent) = target.parent() {
                if !parent.exists() {
                    std::fs::create_dir_all(parent)?;
                }
            }
            
            // Copy the file
            std::fs::copy(path, &target)
                .map_err(|e| anyhow::anyhow!("Failed to copy {} to {}: {}", path.display(), target.display(), e))?;
            
            println!("Copied: {} -> {}", path.display(), target.display());
        } else if path.is_dir() && !target.exists() {
            // Create the directory if it doesn't exist
            std::fs::create_dir_all(&target)
                .map_err(|e| anyhow::anyhow!("Failed to create directory {}: {}", target.display(), e))?;
        }
    }
    
    println!("Successfully copied {} to {}", src.display(), dst.display());
    
    Ok(())
}

#[allow(dead_code)]
pub fn copy_dir_contents(from: &Path, to: &Path) -> anyhow::Result<()> {
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let from_path = entry.path();
        let to_path = to.join(entry.file_name());

        if file_type.is_dir() {
            create_directory(&to_path)?;
            copy_dir_contents(&from_path, &to_path)?;
        } else {
            fs::copy(&from_path, &to_path)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn test_create_directory() -> anyhow::Result<()> {
        // Create a temporary directory that will be automatically removed when the test completes
        let temp_dir = tempdir()?;
        let test_dir_path = temp_dir.path().join("test_directory");
        
        // Verify the directory doesn't exist yet
        assert!(!test_dir_path.exists());
        
        // Create the directory
        create_directory(&test_dir_path)?;
        
        // Verify the directory was created
        assert!(test_dir_path.exists());
        assert!(test_dir_path.is_dir());
        
        // Test creating a directory that already exists
        create_directory(&test_dir_path)?;
        
        // Test creating nested directories
        let nested_dir_path = test_dir_path.join("nested").join("directories");
        create_directory(&nested_dir_path)?;
        assert!(nested_dir_path.exists());
        assert!(nested_dir_path.is_dir());
        
        Ok(())
    }
    
    #[test]
    fn test_copy_directory() -> anyhow::Result<()> {
        // Create a temporary directory that will be automatically removed when the test completes
        let temp_dir = tempdir()?;
        
        // Create source directory structure
        let src_dir = temp_dir.path().join("src_dir");
        create_directory(&src_dir)?;
        
        // Create some files and subdirectories in the source directory
        let file1_path = src_dir.join("file1.txt");
        let mut file1 = File::create(&file1_path)?;
        writeln!(file1, "This is file 1 content")?;
        
        let subdir_path = src_dir.join("subdir");
        create_directory(&subdir_path)?;
        
        let file2_path = subdir_path.join("file2.txt");
        let mut file2 = File::create(&file2_path)?;
        writeln!(file2, "This is file 2 content in a subdirectory")?;
        
        // Create destination directory
        let dst_dir = temp_dir.path().join("dst_dir");
        
        // Copy the directory
        copy_directory(&src_dir, &dst_dir)?;
        
        // Verify the destination directory was created
        assert!(dst_dir.exists());
        assert!(dst_dir.is_dir());
        
        // Verify files were copied correctly
        let dst_file1 = dst_dir.join("file1.txt");
        assert!(dst_file1.exists());
        assert!(dst_file1.is_file());
        
        let dst_subdir = dst_dir.join("subdir");
        assert!(dst_subdir.exists());
        assert!(dst_subdir.is_dir());
        
        let dst_file2 = dst_subdir.join("file2.txt");
        assert!(dst_file2.exists());
        assert!(dst_file2.is_file());
        
        // Verify file contents
        let content1 = fs::read_to_string(&dst_file1)?;
        assert_eq!(content1, "This is file 1 content\n");
        
        let content2 = fs::read_to_string(&dst_file2)?;
        assert_eq!(content2, "This is file 2 content in a subdirectory\n");
        
        Ok(())
    }
    
    #[test]
    fn test_copy_dir_contents() -> anyhow::Result<()> {
        // Create a temporary directory that will be automatically removed when the test completes
        let temp_dir = tempdir()?;
        
        // Create source directory with contents
        let src_dir = temp_dir.path().join("src_contents");
        create_directory(&src_dir)?;
        
        // Create some files in the source directory
        let file1_path = src_dir.join("file1.txt");
        let mut file1 = File::create(&file1_path)?;
        writeln!(file1, "Content file 1")?;
        
        let file2_path = src_dir.join("file2.txt");
        let mut file2 = File::create(&file2_path)?;
        writeln!(file2, "Content file 2")?;
        
        // Create a subdirectory with a file
        let subdir_path = src_dir.join("nested");
        create_directory(&subdir_path)?;
        
        let nested_file_path = subdir_path.join("nested_file.txt");
        let mut nested_file = File::create(&nested_file_path)?;
        writeln!(nested_file, "Nested file content")?;
        
        // Create destination directory
        let dst_dir = temp_dir.path().join("dst_contents");
        create_directory(&dst_dir)?;
        
        // Copy the directory contents
        copy_dir_contents(&src_dir, &dst_dir)?;
        
        // Verify files were copied correctly
        let dst_file1 = dst_dir.join("file1.txt");
        assert!(dst_file1.exists());
        assert!(dst_file1.is_file());
        
        let dst_file2 = dst_dir.join("file2.txt");
        assert!(dst_file2.exists());
        assert!(dst_file2.is_file());
        
        // Verify subdirectory and its contents were copied
        let dst_subdir = dst_dir.join("nested");
        assert!(dst_subdir.exists());
        assert!(dst_subdir.is_dir());
        
        let dst_nested_file = dst_subdir.join("nested_file.txt");
        assert!(dst_nested_file.exists());
        assert!(dst_nested_file.is_file());
        
        // Verify file contents
        let content1 = fs::read_to_string(&dst_file1)?;
        assert_eq!(content1, "Content file 1\n");
        
        let content2 = fs::read_to_string(&dst_file2)?;
        assert_eq!(content2, "Content file 2\n");
        
        let nested_content = fs::read_to_string(&dst_nested_file)?;
        assert_eq!(nested_content, "Nested file content\n");
        
        Ok(())
    }
}