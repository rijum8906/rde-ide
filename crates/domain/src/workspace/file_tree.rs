//! File tree domain module: directory scanning and hierarchical item generation.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::Result;

/// A single node in the file system.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileTreeNode {
    pub path: PathBuf,
    pub name: String,
    pub is_dir: bool,
}

/// A flattened item representing a visible line in the file tree view.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileTreeItem {
    pub path: PathBuf,
    pub name: String,
    pub depth: usize,
    pub is_dir: bool,
    pub is_expanded: bool,
}

/// Scans a directory on disk and returns its child entries sorted.
/// Directories come first, followed by files in alphabetical order.
pub fn scan_dir(path: &Path) -> Result<Vec<FileTreeNode>> {
    let mut entries = Vec::new();

    if let Ok(read_dir) = fs::read_dir(path) {
        for entry in read_dir.flatten() {
            let file_type = match entry.file_type() {
                Ok(ft) => ft,
                Err(_) => continue,
            };

            let name = entry.file_name().to_string_lossy().to_string();

            // Skip hidden files/directories except .gitignore
            if name.starts_with('.') && name != ".gitignore" {
                continue;
            }

            // Skip common build artifacts
            if name == "target" || name == "node_modules" {
                continue;
            }

            entries.push(FileTreeNode {
                path: entry.path(),
                name,
                is_dir: file_type.is_dir(),
            });
        }
    }

    // Sort: directories first, then alphabetical (case-insensitive)
    entries.sort_by(|a, b| match (a.is_dir, b.is_dir) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
    });

    Ok(entries)
}

/// Recursively builds a flattened list of visible items for a file tree given
/// a root path and a set of expanded directory paths.
pub fn build_visible_tree(
    root: &Path,
    expanded_dirs: &HashSet<PathBuf>,
) -> Result<Vec<FileTreeItem>> {
    let mut items = Vec::new();
    populate_tree(root, 0, expanded_dirs, &mut items, 0)?;
    Ok(items)
}

fn populate_tree(
    dir: &Path,
    depth: usize,
    expanded_dirs: &HashSet<PathBuf>,
    out: &mut Vec<FileTreeItem>,
    recursion_depth: usize,
) -> Result<()> {
    // Avoid unbounded recursion on cyclic symlinks
    if recursion_depth > 12 {
        return Ok(());
    }

    let entries = scan_dir(dir)?;

    for entry in entries {
        let is_expanded = entry.is_dir && expanded_dirs.contains(&entry.path);

        out.push(FileTreeItem {
            path: entry.path.clone(),
            name: entry.name,
            depth,
            is_dir: entry.is_dir,
            is_expanded,
        });

        if is_expanded {
            populate_tree(
                &entry.path,
                depth + 1,
                expanded_dirs,
                out,
                recursion_depth + 1,
            )?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::tempdir;

    #[test]
    fn test_scan_dir_sorting() {
        let dir = tempdir().unwrap();
        let root = dir.path();

        File::create(root.join("b_file.txt")).unwrap();
        File::create(root.join("a_file.txt")).unwrap();
        fs::create_dir(root.join("z_dir")).unwrap();
        fs::create_dir(root.join("a_dir")).unwrap();

        let entries = scan_dir(root).unwrap();

        // Directories first
        assert!(entries[0].is_dir);
        assert_eq!(entries[0].name, "a_dir");
        assert!(entries[1].is_dir);
        assert_eq!(entries[1].name, "z_dir");

        // Files next
        assert!(!entries[2].is_dir);
        assert_eq!(entries[2].name, "a_file.txt");
        assert!(!entries[3].is_dir);
        assert_eq!(entries[3].name, "b_file.txt");
    }

    #[test]
    fn test_build_visible_tree_expansion() {
        let dir = tempdir().unwrap();
        let root = dir.path();

        let sub_dir = root.join("sub");
        fs::create_dir(&sub_dir).unwrap();
        File::create(sub_dir.join("inner.rs")).unwrap();

        // When collapsed, inner.rs is not in the visible tree
        let expanded = HashSet::new();
        let items = build_visible_tree(root, &expanded).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].name, "sub");
        assert!(!items[0].is_expanded);

        // When expanded, inner.rs appears with depth 1
        let mut expanded = HashSet::new();
        expanded.insert(sub_dir);
        let items = build_visible_tree(root, &expanded).unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].name, "sub");
        assert!(items[0].is_expanded);
        assert_eq!(items[1].name, "inner.rs");
        assert_eq!(items[1].depth, 1);
    }
}
