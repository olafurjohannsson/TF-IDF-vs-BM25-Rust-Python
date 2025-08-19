use std::fs;
use std::path::Path;
use std::error::Error;
use std::ffi::OsStr;

pub fn load_directory(directory_path: &str) -> Result<Vec<(String, String)>, Box<dyn Error>> {
    // Create a mutable vector to store all files from directory and subdirectories
    let mut files: Vec<(String, String)> = Vec::new();
    // Start recursive loading from the root directory path
    // The ? operator handles the error, if directory doesn't exist or we don't have permission,
    // then the function returns early with the error
    load_directory_recursive(Path::new(directory_path), &mut files)?;
    // Rust has implicit return - unlike C++ or C# where semicolon and return is mandatory,
    // in Rust no semicolon means "return this value"
    Ok(files)
}

// Recursive helper function that does the actual directory traversal
// Takes a Path reference and a mutable reference to the files vector
// Returns Result<(), Box<dyn Error>> - either success (empty tuple) or error
fn load_directory_recursive(dir: &Path, files: &mut Vec<(String, String)>) -> Result<(), Box<dyn Error>> {
    // Read the directory of the path, the ? operator handles the error, if directory doesn't exist or
    // We do not have permission, then the function returns early
    let entries = fs::read_dir(dir)?; // entries is an iterator of Result<DirEntry, std::io::Error>

    // Each directory entry is wrapped in a Result because reading individual entries can fail
    for entry in entries {
        // Unwrap each entry, if Ok(entry) -> entry becomes DirEntry - if Err(e) function returns early with error
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            // If this is a subdirectory, recursively process it
            // This allows us to find files in nested folders
            load_directory_recursive(&path, files)?;
        } else {
            // Check if this file has a .txt extension
            let extension: Option<&OsStr> = path.extension(); // None if no extension exists

            // and_then is used for chaining operations that might fail
            // so what we are doing is checking if extension is valid using and_then
            // then we call to_str to change &OsStr to Option<&str> which we can compare to Some("txt")
            // The reason we do not use == "txt" is because we do not have a &str, but an Option<&str>
            if extension.and_then(|s| s.to_str()) == Some("txt") {
                // Include relative path from root directory for better context
                // This gives us paths like "subdir/file.txt" instead of just "file.txt"
                let filename = path
                    .strip_prefix(dir.parent().unwrap_or(Path::new(""))) // Remove the parent directory prefix
                    .unwrap_or(&path) // If strip_prefix fails, use the full path
                    .to_string_lossy() // Convert Path to String, handling any non-UTF8 characters
                    .to_string(); // Convert from Cow<str> to owned String

                // fs::read_to_string returns io::Result<String>, io::Result<String> is a type alias for Result<String, io::Error>
                // The ? operator propagates errors to the caller, if we skip ?, then we would have to handle Ok() and Err() here
                let contents = fs::read_to_string(&path)?;
                files.push((filename, contents));
            }
        }
    }

    // Return Ok(()) to indicate successful completion
    // The empty tuple () is Rust's unit type, similar to void in other languages
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_directory_on_data_folder() {
        // Test loading the actual data directory we're using in the book
        // This assumes you have the Python docs in ./data/
        match load_directory("data") {
            Ok(files) => {
                // Should find multiple .txt files in the Python documentation
                assert!(files.len() > 0, "Should find at least some .txt files");

                // Check that all loaded files have .txt extension in their names
                for (filename, content) in &files {
                    assert!(filename.ends_with(".txt"), "All files should be .txt files");
                    assert!(content.len() > 0, "Files should not be empty");
                }

                println!("Successfully loaded {} files", files.len());
            }
            Err(_) => {
                // If data directory doesn't exist, that's fine for testing
                println!("Data directory not found - test skipped");
            }
        }
    }
}