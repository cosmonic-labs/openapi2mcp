use crate::Result;
use crate::backend::FileBackend;
use std::fs;
use std::path::Path;

/// Native filesystem backend using std::fs
pub struct NativeFileBackend;

impl FileBackend for NativeFileBackend {
    fn read_file(&self, path: &str) -> Result<String> {
        Ok(fs::read_to_string(path)?)
    }

    fn write_file(&self, path: &str, content: &str) -> Result<()> {
        if let Some(parent) = Path::new(path).parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, content)?;
        Ok(())
    }

    fn create_dir_all(&self, path: &str) -> Result<()> {
        fs::create_dir_all(path)?;
        Ok(())
    }

    fn exists(&self, path: &str) -> bool {
        Path::new(path).exists()
    }

    fn is_dir(&self, path: &str) -> bool {
        Path::new(path).is_dir()
    }

    fn list_dir(&self, path: &str) -> Result<Vec<String>> {
        let entries = fs::read_dir(path)?
            .map(|res| res.map(|e| e.file_name().to_string_lossy().to_string()))
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(entries)
    }

    fn remove_file(&self, path: &str) -> Result<()> {
        fs::remove_file(path)?;
        Ok(())
    }

    fn remove_dir_all(&self, path: &str) -> Result<()> {
        fs::remove_dir_all(path)?;
        Ok(())
    }

    fn copy_file(&self, src: &str, dest: &str) -> Result<()> {
        if let Some(parent) = Path::new(dest).parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(src, dest)?;
        Ok(())
    }
}

/// Helper function to prepare TypeScript template (separate step)
pub fn prepare_typescript_template(template_dir: &str) -> Result<()> {
    use std::process::Command;

    if !Path::new(template_dir).exists() {
        log::info!(
            "Cloning TypeScript template from GitHub to {}",
            template_dir
        );
        let status = Command::new("git")
            .args([
                "clone",
                "https://github.com/cosmonic-labs/mcp-server-template-ts",
                template_dir,
            ])
            .status()?;

        if !status.success() {
            return Err(crate::Error::Validation(
                "Failed to clone mcp-server-template-ts repository".to_string(),
            ));
        }
    }
    Ok(())
}
