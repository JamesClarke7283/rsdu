use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct FileInfo {
    pub path: PathBuf,
    pub size: u64,
    pub is_dir: bool,
    pub children: Option<Vec<FileInfo>>,
    pub items: u64, // Added this field
}

impl FileInfo {
    /// Creates a new `FileInfo` instance.
    pub fn new(path: PathBuf, size: u64, is_dir: bool, items: u64) -> Self {
        FileInfo {
            path,
            size,
            is_dir,
            children: None,
            items,
        }
    }
}

/// Recursively traverses a directory and calculates the size of each file and directory.
pub fn traverse_directory(path: &Path) -> io::Result<FileInfo> {
    let metadata = fs::metadata(path)?;
    let is_dir = metadata.is_dir();
    let mut size = 0;
    let mut items = 1; // Count the current item
    let mut children = Vec::new();

    if is_dir {
        let read_dir = fs::read_dir(path)?;
        for entry_result in read_dir {
            let entry = entry_result?;
            let child_path = entry.path();

            match traverse_directory(&child_path) {
                Ok(child_info) => {
                    size += child_info.size;
                    items += child_info.items;
                    children.push(child_info);
                }
                Err(e) => {
                    eprintln!("Warning: Could not traverse {}: {}", child_path.display(), e);
                    continue;
                }
            }
        }
    } else {
        size = metadata.len();
    }

    let mut file_info = FileInfo::new(path.to_path_buf(), size, is_dir, items);
    if is_dir {
        file_info.children = Some(children);
    }

    Ok(file_info)
}
