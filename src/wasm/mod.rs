use crate::Result;
use crate::backend::FileBackend;

mod plugin;

pub(crate) mod bindings {
    use super::plugin::Plugin;
    wit_bindgen::generate!({
        world: "plugin-guest",
        with: {
            "wasi:io/streams@0.2.0": ::wasi::io::streams
        },
        generate_all
    });

    export!(Plugin);
}

/// WASI-compatible FileBackend using wasi:filesystem
pub struct WasiFileBackend;

impl WasiFileBackend {
    pub fn new() -> Self {
        Self
    }

    /// Get the root directory from WASI preopened directories
    fn get_root_dir() -> Result<(bindings::wasi::filesystem::types::Descriptor, String)> {
        use bindings::wasi::logging::logging::{Level, log};

        log(
            Level::Debug,
            "wasi-fs",
            "Getting WASI preopened directories",
        );
        let mut dirs = bindings::wasi::filesystem::preopens::get_directories();
        log(
            Level::Debug,
            "wasi-fs",
            &format!("Found {} preopened directories", dirs.len()),
        );

        if let Some(dir) = dirs.pop() {
            log(
                Level::Debug,
                "wasi-fs",
                &format!("Using preopened directory: {}", dir.1),
            );
            Ok((dir.0, dir.1))
        } else {
            log(Level::Error, "wasi-fs", "No preopened directories found");
            Err(crate::Error::Validation(
                "No preopened directories found".to_string(),
            ))
        }
    }

    /// Convert path to directory and filename components
    fn split_path(path: &str) -> (Option<&str>, &str) {
        if let Some(pos) = path.rfind('/') {
            let (dir, file) = path.split_at(pos);
            (Some(dir), &file[1..]) // Skip the '/' character
        } else {
            (None, path)
        }
    }
}

impl FileBackend for WasiFileBackend {
    fn read_file(&self, path: &str) -> Result<String> {
        use bindings::wasi::filesystem::types::{DescriptorFlags, OpenFlags, PathFlags};
        use bindings::wasi::logging::logging::{Level, log};
        use std::io::Read;

        log(Level::Debug, "wasi-fs", &format!("Reading file: {}", path));
        let (root_dir, _root_path) = Self::get_root_dir()?;

        log(
            Level::Debug,
            "wasi-fs",
            &format!("Opening file at path: {}", path),
        );
        let file = root_dir
            .open_at(
                PathFlags::empty(),
                path,
                OpenFlags::empty(),
                DescriptorFlags::READ,
            )
            .map_err(|e| {
                log(
                    Level::Error,
                    "wasi-fs",
                    &format!("Failed to open file {}: {}", path, e),
                );
                crate::Error::Validation(format!("Failed to open file {}: {}", path, e))
            })?;

        log(Level::Debug, "wasi-fs", "Creating read stream");
        let mut stream = file.read_via_stream(0).map_err(|e| {
            log(
                Level::Error,
                "wasi-fs",
                &format!("Failed to create read stream: {}", e),
            );
            crate::Error::Validation(format!("Failed to create read stream: {}", e))
        })?;

        log(Level::Debug, "wasi-fs", "Reading content from stream");
        let mut content = String::new();
        stream.read_to_string(&mut content).map_err(|e| {
            log(
                Level::Error,
                "wasi-fs",
                &format!("Failed to read content: {}", e),
            );
            crate::Error::Io(e)
        })?;

        log(
            Level::Debug,
            "wasi-fs",
            &format!("Successfully read {} bytes", content.len()),
        );
        Ok(content)
    }

    fn write_file(&self, path: &str, content: &str) -> Result<()> {
        use bindings::wasi::filesystem::types::{DescriptorFlags, OpenFlags, PathFlags};
        use bindings::wasi::logging::logging::{Level, log};

        log(
            Level::Debug,
            "wasi-fs",
            &format!("Writing file: {} ({} bytes)", path, content.len()),
        );
        let (root_dir, _root_path) = Self::get_root_dir()?;

        // Create parent directories if needed
        if let (Some(parent_dir), _) = Self::split_path(path) {
            log(
                Level::Debug,
                "wasi-fs",
                &format!("Creating parent directories for: {}", parent_dir),
            );
            self.create_dir_all(parent_dir)?;
        }

        log(
            Level::Debug,
            "wasi-fs",
            &format!("Opening file for writing: {}", path),
        );
        let file = root_dir
            .open_at(
                PathFlags::empty(),
                path,
                OpenFlags::CREATE | OpenFlags::TRUNCATE,
                DescriptorFlags::WRITE,
            )
            .map_err(|e| {
                log(
                    Level::Error,
                    "wasi-fs",
                    &format!("Failed to open file for writing {}: {}", path, e),
                );
                crate::Error::Validation(format!("Failed to open file for writing {}: {}", path, e))
            })?;

        log(Level::Debug, "wasi-fs", "Creating write stream");
        let mut stream = file.write_via_stream(0).map_err(|e| {
            log(
                Level::Error,
                "wasi-fs",
                &format!("Failed to create write stream: {}", e),
            );
            crate::Error::Validation(format!("Failed to create write stream: {}", e))
        })?;

        log(Level::Debug, "wasi-fs", "Copying content to stream");
        let mut content_reader = std::io::Cursor::new(content.as_bytes());
        let bytes_written = std::io::copy(&mut content_reader, &mut stream).map_err(|e| {
            log(
                Level::Error,
                "wasi-fs",
                &format!("Failed to copy content: {}", e),
            );
            crate::Error::Io(e)
        })?;

        log(
            Level::Debug,
            "wasi-fs",
            &format!("Copied {} bytes to stream", bytes_written),
        );

        log(Level::Debug, "wasi-fs", "Blocking flush write stream");
        stream.blocking_flush().map_err(|e| {
            log(
                Level::Error,
                "wasi-fs",
                &format!("Failed to blocking flush stream: {:?}", e),
            );
            crate::Error::Validation(format!("Failed to blocking flush stream: {:?}", e))
        })?;

        log(
            Level::Debug,
            "wasi-fs",
            "Dropping write stream to ensure close",
        );
        drop(stream);
        log(
            Level::Debug,
            "wasi-fs",
            "Dropping file descriptor to ensure close",
        );
        drop(file);

        log(
            Level::Debug,
            "wasi-fs",
            &format!("Successfully wrote file: {}", path),
        );
        Ok(())
    }

    fn create_dir_all(&self, path: &str) -> Result<()> {
        use bindings::wasi::logging::logging::{Level, log};

        log(
            Level::Debug,
            "wasi-fs",
            &format!("Creating directories for path: {}", path),
        );
        let (root_dir, _root_path) = Self::get_root_dir()?;

        // Split path into components and create each directory
        let parts: Vec<&str> = path.split('/').filter(|p| !p.is_empty()).collect();
        log(
            Level::Debug,
            "wasi-fs",
            &format!("Path components: {:?}", parts),
        );
        let mut current_path = String::new();

        for part in parts {
            if !current_path.is_empty() {
                current_path.push('/');
            }
            current_path.push_str(part);

            log(
                Level::Debug,
                "wasi-fs",
                &format!("Creating directory: {}", current_path),
            );
            // Try to create the directory, ignore if it already exists
            if let Err(e) = root_dir.create_directory_at(&current_path) {
                if e != bindings::wasi::filesystem::types::ErrorCode::Exist {
                    log(
                        Level::Error,
                        "wasi-fs",
                        &format!("Failed to create directory {}: {}", current_path, e),
                    );
                    return Err(crate::Error::Validation(format!(
                        "Failed to create directory {}: {}",
                        current_path, e
                    )));
                } else {
                    log(
                        Level::Debug,
                        "wasi-fs",
                        &format!("Directory already exists: {}", current_path),
                    );
                }
            } else {
                log(
                    Level::Debug,
                    "wasi-fs",
                    &format!("Successfully created directory: {}", current_path),
                );
            }
        }

        log(
            Level::Debug,
            "wasi-fs",
            &format!("Finished creating directories for: {}", path),
        );
        Ok(())
    }

    fn exists(&self, path: &str) -> bool {
        use bindings::wasi::filesystem::types::PathFlags;
        use bindings::wasi::logging::logging::{Level, log};

        log(
            Level::Debug,
            "wasi-fs",
            &format!("Checking if path exists: {}", path),
        );
        let Ok((root_dir, _root_path)) = Self::get_root_dir() else {
            log(
                Level::Error,
                "wasi-fs",
                "Failed to get root directory for exists check",
            );
            return false;
        };

        let exists = root_dir.stat_at(PathFlags::empty(), path).is_ok();
        log(
            Level::Debug,
            "wasi-fs",
            &format!("Path {} exists: {}", path, exists),
        );
        exists
    }

    fn is_dir(&self, path: &str) -> bool {
        use bindings::wasi::filesystem::types::{DescriptorType, PathFlags};
        use bindings::wasi::logging::logging::{Level, log};

        log(
            Level::Debug,
            "wasi-fs",
            &format!("Checking if path is directory: {}", path),
        );
        let Ok((root_dir, _root_path)) = Self::get_root_dir() else {
            log(
                Level::Error,
                "wasi-fs",
                "Failed to get root directory for is_dir check",
            );
            return false;
        };

        match root_dir.stat_at(PathFlags::empty(), path) {
            Ok(stat) => {
                let is_directory = stat.type_ == DescriptorType::Directory;
                log(
                    Level::Debug,
                    "wasi-fs",
                    &format!("Path {} is directory: {}", path, is_directory),
                );
                is_directory
            }
            Err(e) => {
                log(
                    Level::Error,
                    "wasi-fs",
                    &format!("Failed to stat path {}: {}", path, e),
                );
                false
            }
        }
    }

    fn list_dir(&self, path: &str) -> Result<Vec<String>> {
        use bindings::wasi::filesystem::types::{DescriptorFlags, OpenFlags, PathFlags};
        use bindings::wasi::logging::logging::{Level, log};

        log(
            Level::Debug,
            "wasi-fs",
            &format!("Listing directory: {}", path),
        );
        let (root_dir, _root_path) = Self::get_root_dir()?;

        log(
            Level::Debug,
            "wasi-fs",
            &format!("Opening directory: {}", path),
        );
        let dir = root_dir
            .open_at(
                PathFlags::empty(),
                path,
                OpenFlags::DIRECTORY,
                DescriptorFlags::empty(),
            )
            .map_err(|e| {
                log(
                    Level::Error,
                    "wasi-fs",
                    &format!("Failed to open directory {}: {}", path, e),
                );
                crate::Error::Validation(format!("Failed to open directory {}: {}", path, e))
            })?;

        log(Level::Debug, "wasi-fs", "Creating directory stream");
        let dir_stream = dir.read_directory().map_err(|e| {
            log(
                Level::Error,
                "wasi-fs",
                &format!("Failed to read directory: {}", e),
            );
            crate::Error::Validation(format!("Failed to read directory: {}", e))
        })?;

        log(Level::Debug, "wasi-fs", "Reading directory entries");
        let mut entries = Vec::new();
        while let Ok(Some(entry)) = dir_stream.read_directory_entry() {
            log(
                Level::Debug,
                "wasi-fs",
                &format!("Found directory entry: {:?}", entry),
            );
            entries.push(entry.name);
        }

        log(
            Level::Debug,
            "wasi-fs",
            &format!("Found {} entries in directory {}", entries.len(), path),
        );
        Ok(entries)
    }

    fn remove_file(&self, path: &str) -> Result<()> {
        use bindings::wasi::filesystem::types::{DescriptorFlags, OpenFlags, PathFlags};
        use bindings::wasi::logging::logging::{Level, log};

        log(Level::Debug, "wasi-fs", &format!("Removing file: {}", path));
        let (root_dir, _root_path) = Self::get_root_dir()?;

        if let (Some(parent_dir), filename) = Self::split_path(path) {
            log(
                Level::Debug,
                "wasi-fs",
                &format!("Opening parent directory: {}", parent_dir),
            );
            let parent = root_dir
                .open_at(
                    PathFlags::empty(),
                    parent_dir,
                    OpenFlags::DIRECTORY,
                    DescriptorFlags::MUTATE_DIRECTORY,
                )
                .map_err(|e| {
                    log(
                        Level::Error,
                        "wasi-fs",
                        &format!("Failed to open parent directory: {}", e),
                    );
                    crate::Error::Validation(format!("Failed to open parent directory: {}", e))
                })?;

            log(
                Level::Debug,
                "wasi-fs",
                &format!("Unlinking file: {}", filename),
            );
            parent.unlink_file_at(filename).map_err(|e| {
                log(
                    Level::Error,
                    "wasi-fs",
                    &format!("Failed to remove file {}: {}", path, e),
                );
                crate::Error::Validation(format!("Failed to remove file {}: {}", path, e))
            })?;
        } else {
            // File is in root directory
            log(
                Level::Debug,
                "wasi-fs",
                &format!("Removing file from root: {}", path),
            );
            root_dir.unlink_file_at(path).map_err(|e| {
                log(
                    Level::Error,
                    "wasi-fs",
                    &format!("Failed to remove file {}: {}", path, e),
                );
                crate::Error::Validation(format!("Failed to remove file {}: {}", path, e))
            })?;
        }

        log(
            Level::Debug,
            "wasi-fs",
            &format!("Successfully removed file: {}", path),
        );
        Ok(())
    }

    fn remove_dir_all(&self, path: &str) -> Result<()> {
        use bindings::wasi::filesystem::types::{
            DescriptorFlags, DescriptorType, OpenFlags, PathFlags,
        };
        use bindings::wasi::logging::logging::{Level, log};

        log(
            Level::Debug,
            "wasi-fs",
            &format!("Removing directory recursively: {}", path),
        );
        let (root_dir, _root_path) = Self::get_root_dir()?;

        log(
            Level::Debug,
            "wasi-fs",
            &format!("Opening directory for removal: {}", path),
        );
        let dir = root_dir
            .open_at(
                PathFlags::empty(),
                path,
                OpenFlags::DIRECTORY,
                DescriptorFlags::MUTATE_DIRECTORY,
            )
            .map_err(|e| {
                log(
                    Level::Error,
                    "wasi-fs",
                    &format!("Failed to open directory {}: {}", path, e),
                );
                crate::Error::Validation(format!("Failed to open directory {}: {}", path, e))
            })?;

        // Recursively remove contents
        log(
            Level::Debug,
            "wasi-fs",
            "Reading directory contents for removal",
        );
        let dir_stream = dir.read_directory().map_err(|e| {
            log(
                Level::Error,
                "wasi-fs",
                &format!("Failed to read directory: {}", e),
            );
            crate::Error::Validation(format!("Failed to read directory: {}", e))
        })?;

        while let Ok(Some(entry)) = dir_stream.read_directory_entry() {
            let entry_path = format!("{}/{}", path, entry.name);
            log(
                Level::Debug,
                "wasi-fs",
                &format!(
                    "Processing entry for removal: {} (type: {:?})",
                    entry_path, entry.type_
                ),
            );

            if entry.type_ == DescriptorType::Directory {
                log(
                    Level::Debug,
                    "wasi-fs",
                    &format!("Recursively removing directory: {}", entry_path),
                );
                self.remove_dir_all(&entry_path)?;
            } else if entry.type_ == DescriptorType::RegularFile {
                log(
                    Level::Debug,
                    "wasi-fs",
                    &format!("Removing file: {}", entry_path),
                );
                self.remove_file(&entry_path)?;
            }
        }

        // Remove the directory itself
        log(
            Level::Debug,
            "wasi-fs",
            &format!("Removing directory itself: {}", path),
        );
        if let (Some(parent_dir), dirname) = Self::split_path(path) {
            log(
                Level::Debug,
                "wasi-fs",
                &format!("Opening parent directory: {}", parent_dir),
            );
            let parent = root_dir
                .open_at(
                    PathFlags::empty(),
                    parent_dir,
                    OpenFlags::DIRECTORY,
                    DescriptorFlags::MUTATE_DIRECTORY,
                )
                .map_err(|e| {
                    log(
                        Level::Error,
                        "wasi-fs",
                        &format!("Failed to open parent directory: {}", e),
                    );
                    crate::Error::Validation(format!("Failed to open parent directory: {}", e))
                })?;

            log(
                Level::Debug,
                "wasi-fs",
                &format!("Removing directory: {}", dirname),
            );
            parent.remove_directory_at(dirname).map_err(|e| {
                log(
                    Level::Error,
                    "wasi-fs",
                    &format!("Failed to remove directory {}: {}", path, e),
                );
                crate::Error::Validation(format!("Failed to remove directory {}: {}", path, e))
            })?;
        } else {
            log(
                Level::Debug,
                "wasi-fs",
                &format!("Removing directory from root: {}", path),
            );
            root_dir.remove_directory_at(path).map_err(|e| {
                log(
                    Level::Error,
                    "wasi-fs",
                    &format!("Failed to remove directory {}: {}", path, e),
                );
                crate::Error::Validation(format!("Failed to remove directory {}: {}", path, e))
            })?;
        }

        log(
            Level::Debug,
            "wasi-fs",
            &format!("Successfully removed directory: {}", path),
        );
        Ok(())
    }

    fn copy_file(&self, src: &str, dest: &str) -> Result<()> {
        use bindings::wasi::logging::logging::{Level, log};

        log(
            Level::Debug,
            "wasi-fs",
            &format!("Copying file from {} to {}", src, dest),
        );
        let content = self.read_file(src)?;
        self.write_file(dest, &content)?;
        log(
            Level::Debug,
            "wasi-fs",
            &format!("Successfully copied file from {} to {}", src, dest),
        );
        Ok(())
    }
}

/// WASM-specific functionality for generating MCP projects
pub mod generator {
    use super::WasiFileBackend;
    use crate::cli::Target;
    use crate::mcp::McpGenerator;
    use crate::openapi::OpenApiSpec;

    /// Generate MCP project using WASI filesystem (WASM entry point)
    pub fn generate_mcp_project(
        spec: OpenApiSpec,
        language: Target,
        template_dir: &str,
        output_dir: &str,
        server_name: Option<&str>,
    ) -> crate::Result<()> {
        use super::bindings::wasi::logging::logging::{Level, log};

        log(
            Level::Debug,
            "wasi-fs",
            &format!(
                "Starting MCP generation with template_dir: {}, output_dir: {}, language: {:?}",
                template_dir, output_dir, language
            ),
        );

        if let Some(name) = server_name {
            log(Level::Debug, "wasi-fs", &format!("Server name: {}", name));
        }

        let backend = WasiFileBackend::new();
        log(Level::Debug, "wasi-fs", "Created WASI file backend");

        let generator = McpGenerator::new(spec, language);
        log(Level::Debug, "wasi-fs", "Created MCP generator");

        log(Level::Debug, "wasi-fs", "Starting generation process");
        let result = generator.generate(&backend, template_dir, output_dir, server_name);

        match &result {
            Ok(_) => log(
                Level::Debug,
                "wasi-fs",
                "MCP generation completed successfully",
            ),
            Err(e) => log(
                Level::Error,
                "wasi-fs",
                &format!("MCP generation failed: {}", e),
            ),
        }

        result
    }
}
