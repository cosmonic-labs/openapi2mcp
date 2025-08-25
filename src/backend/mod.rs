use crate::Result;

/// Minimal filesystem abstraction for MCP generation
pub trait FileBackend {
    /// Read a file as a string
    fn read_file(&self, path: &str) -> Result<String>;

    /// Write a file with given content  
    fn write_file(&self, path: &str, content: &str) -> Result<()>;

    /// Create a directory and any parent directories
    fn create_dir_all(&self, path: &str) -> Result<()>;

    /// Check if a path exists
    fn exists(&self, path: &str) -> bool;

    /// Check if a path is a directory
    fn is_dir(&self, path: &str) -> bool;

    /// List entries in a directory (returns entry names only)
    fn list_dir(&self, path: &str) -> Result<Vec<String>>;

    /// Remove a file
    fn remove_file(&self, path: &str) -> Result<()>;

    /// Remove a directory and all its contents
    fn remove_dir_all(&self, path: &str) -> Result<()>;

    /// Copy a file from source to destination
    fn copy_file(&self, src: &str, dest: &str) -> Result<()>;
}

pub mod native;
