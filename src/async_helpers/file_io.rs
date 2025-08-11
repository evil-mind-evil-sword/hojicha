//! File I/O helper commands

use crate::core::{Cmd, Message};
use crate::commands;
use std::path::{Path, PathBuf};

/// File operation errors
#[derive(Debug, Clone)]
pub enum FileError {
    /// File not found
    NotFound(PathBuf),
    /// Permission denied
    PermissionDenied(PathBuf),
    /// I/O error
    IoError(String),
    /// UTF-8 decoding error
    Utf8Error(String),
}

impl std::fmt::Display for FileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileError::NotFound(path) => write!(f, "File not found: {}", path.display()),
            FileError::PermissionDenied(path) => write!(f, "Permission denied: {}", path.display()),
            FileError::IoError(e) => write!(f, "I/O error: {}", e),
            FileError::Utf8Error(e) => write!(f, "UTF-8 error: {}", e),
        }
    }
}

impl std::error::Error for FileError {}

/// File change event for file watching
#[derive(Debug, Clone)]
pub enum FileEvent {
    /// File was created
    Created(PathBuf),
    /// File was modified
    Modified(PathBuf),
    /// File was deleted
    Deleted(PathBuf),
    /// File was renamed
    Renamed { from: PathBuf, to: PathBuf },
}

/// Read a file asynchronously
///
/// # Example
/// ```no_run
/// # use hojicha_core::async_helpers::read_file;
/// # #[derive(Clone)]
/// # enum Msg {
/// #     FileLoaded(String),
/// #     Error(String),
/// # }
/// 
/// read_file("config.json", |result| {
///     match result {
///         Ok(content) => Msg::FileLoaded(content),
///         Err(e) => Msg::Error(e.to_string()),
///     }
/// })
/// # ;
/// ```
pub fn read_file<M, F, P>(path: P, handler: F) -> Cmd<M>
where
    M: Message,
    F: FnOnce(Result<String, FileError>) -> M + Send + 'static,
    P: AsRef<Path> + Send + 'static,
{
    let path = path.as_ref().to_path_buf();
    
    commands::spawn(async move {
        let result = tokio::fs::read_to_string(&path).await
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    FileError::NotFound(path.clone())
                } else if e.kind() == std::io::ErrorKind::PermissionDenied {
                    FileError::PermissionDenied(path.clone())
                } else {
                    FileError::IoError(e.to_string())
                }
            });
        
        Some(handler(result))
    })
}

/// Read a file as bytes
pub fn read_file_bytes<M, F, P>(path: P, handler: F) -> Cmd<M>
where
    M: Message,
    F: FnOnce(Result<Vec<u8>, FileError>) -> M + Send + 'static,
    P: AsRef<Path> + Send + 'static,
{
    let path = path.as_ref().to_path_buf();
    
    commands::spawn(async move {
        let result = tokio::fs::read(&path).await
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    FileError::NotFound(path.clone())
                } else if e.kind() == std::io::ErrorKind::PermissionDenied {
                    FileError::PermissionDenied(path.clone())
                } else {
                    FileError::IoError(e.to_string())
                }
            });
        
        Some(handler(result))
    })
}

/// Write to a file asynchronously
///
/// # Example
/// ```no_run
/// # use hojicha_core::async_helpers::write_file;
/// # #[derive(Clone)]
/// # enum Msg {
/// #     FileSaved,
/// #     Error(String),
/// # }
/// 
/// write_file("output.txt", "Hello, World!", |result| {
///     match result {
///         Ok(()) => Msg::FileSaved,
///         Err(e) => Msg::Error(e.to_string()),
///     }
/// })
/// # ;
/// ```
pub fn write_file<M, F, P, C>(path: P, content: C, handler: F) -> Cmd<M>
where
    M: Message,
    F: FnOnce(Result<(), FileError>) -> M + Send + 'static,
    P: AsRef<Path> + Send + 'static,
    C: AsRef<[u8]> + Send + 'static,
{
    let path = path.as_ref().to_path_buf();
    let content = content.as_ref().to_vec();
    
    commands::spawn(async move {
        let result = tokio::fs::write(&path, content).await
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::PermissionDenied {
                    FileError::PermissionDenied(path.clone())
                } else {
                    FileError::IoError(e.to_string())
                }
            });
        
        Some(handler(result))
    })
}

/// Watch a file for changes
///
/// This creates a file watcher that will send events when the file changes.
/// Note: This is a simplified implementation. A real implementation would use
/// notify or similar crate for actual file system watching.
///
/// # Example
/// ```no_run
/// # use hojicha_core::async_helpers::{watch_file, FileEvent};
/// # #[derive(Clone)]
/// # enum Msg {
/// #     FileChanged(String),
/// # }
/// 
/// watch_file("config.json", |event| {
///     match event {
///         FileEvent::Modified(path) => {
///             Some(Msg::FileChanged(path.display().to_string()))
///         }
///         _ => None,
///     }
/// })
/// # ;
/// ```
pub fn watch_file<M, F, P>(path: P, mut handler: F) -> Cmd<M>
where
    M: Message,
    F: FnMut(FileEvent) -> Option<M> + Send + 'static,
    P: AsRef<Path> + Send + 'static,
{
    let path = path.as_ref().to_path_buf();
    
    commands::spawn(async move {
        // In a real implementation, this would use notify or similar
        // For now, we'll simulate a file change after a delay
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        handler(FileEvent::Modified(path))
    })
}

/// List files in a directory
pub fn list_dir<M, F, P>(path: P, handler: F) -> Cmd<M>
where
    M: Message,
    F: FnOnce(Result<Vec<PathBuf>, FileError>) -> M + Send + 'static,
    P: AsRef<Path> + Send + 'static,
{
    let path = path.as_ref().to_path_buf();
    
    commands::spawn(async move {
        let result = async {
            let mut entries = Vec::new();
            let mut dir = tokio::fs::read_dir(&path).await
                .map_err(|e| {
                    if e.kind() == std::io::ErrorKind::NotFound {
                        FileError::NotFound(path.clone())
                    } else if e.kind() == std::io::ErrorKind::PermissionDenied {
                        FileError::PermissionDenied(path.clone())
                    } else {
                        FileError::IoError(e.to_string())
                    }
                })?;
            
            while let Some(entry) = dir.next_entry().await
                .map_err(|e| FileError::IoError(e.to_string()))? 
            {
                entries.push(entry.path());
            }
            
            Ok(entries)
        }.await;
        
        Some(handler(result))
    })
}

/// Create a directory
pub fn create_dir<M, F, P>(path: P, handler: F) -> Cmd<M>
where
    M: Message,
    F: FnOnce(Result<(), FileError>) -> M + Send + 'static,
    P: AsRef<Path> + Send + 'static,
{
    let path = path.as_ref().to_path_buf();
    
    commands::spawn(async move {
        let result = tokio::fs::create_dir_all(&path).await
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::PermissionDenied {
                    FileError::PermissionDenied(path.clone())
                } else {
                    FileError::IoError(e.to_string())
                }
            });
        
        Some(handler(result))
    })
}

/// Delete a file or directory
pub fn delete<M, F, P>(path: P, handler: F) -> Cmd<M>
where
    M: Message,
    F: FnOnce(Result<(), FileError>) -> M + Send + 'static,
    P: AsRef<Path> + Send + 'static,
{
    let path = path.as_ref().to_path_buf();
    
    commands::spawn(async move {
        let result = if path.is_dir() {
            tokio::fs::remove_dir_all(&path).await
        } else {
            tokio::fs::remove_file(&path).await
        }.map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                FileError::NotFound(path.clone())
            } else if e.kind() == std::io::ErrorKind::PermissionDenied {
                FileError::PermissionDenied(path.clone())
            } else {
                FileError::IoError(e.to_string())
            }
        });
        
        Some(handler(result))
    })
}