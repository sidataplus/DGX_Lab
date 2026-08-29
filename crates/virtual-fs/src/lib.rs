#![forbid(unsafe_code)]

//! In-memory virtual filesystem used by the simulator.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum VfsNode {
    Directory { children: BTreeSet<String> },
    File { content: Vec<u8>, mode: u16 },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VirtualFileSystem {
    nodes: BTreeMap<String, VfsNode>,
    quota_bytes: u64,
}

impl VirtualFileSystem {
    #[must_use]
    pub fn new(quota_bytes: u64) -> Self {
        let mut fs = Self { nodes: BTreeMap::new(), quota_bytes };
        fs.nodes.insert("/".into(), VfsNode::Directory { children: BTreeSet::new() });
        fs
    }

    #[must_use]
    pub fn dgx_default() -> Self {
        let mut fs = Self::new(2 * 1024 * 1024 * 1024);
        for path in [
            "/home",
            "/home/learner",
            "/home/learner/labs",
            "/home/learner/logs",
            "/home/learner/checkpoints",
            "/shared",
            "/datasets",
            "/containers",
            "/checkpoints",
            "/scratch",
        ] {
            fs.mkdir_all(path).expect("built-in path is valid");
        }
        fs.write_file(
            "/home/learner/train.sbatch",
            br#"#!/bin/bash
#SBATCH --job-name=train-h200
#SBATCH --partition=gpu
#SBATCH --gres=gpu:h200:1
#SBATCH --cpus-per-task=8
#SBATCH --mem=64G
#SBATCH --time=00:30:00
#SBATCH --output=logs/%x-%j.out

module load singularity
singularity exec --nv /containers/pytorch-lab.sif \
  python train.py --batch-size 64 --epochs 5
"#,
        )
        .expect("built-in file fits quota");
        fs.write_file("/containers/pytorch-lab.sif", b"DGX-LAB-SIMULATED-IMAGE")
            .expect("built-in image fits quota");
        fs
    }

    pub fn mkdir_all(&mut self, path: &str) -> Result<(), VfsError> {
        let normalized = normalize_path(path)?;
        if normalized == "/" {
            return Ok(());
        }
        let mut current = String::from("/");
        for segment in normalized.trim_start_matches('/').split('/') {
            let next = if current == "/" {
                format!("/{segment}")
            } else {
                format!("{current}/{segment}")
            };
            if !self.nodes.contains_key(&next) {
                self.insert_child(&current, segment)?;
                self.nodes.insert(next.clone(), VfsNode::Directory { children: BTreeSet::new() });
            } else if !matches!(self.nodes.get(&next), Some(VfsNode::Directory { .. })) {
                return Err(VfsError::NotDirectory(next));
            }
            current = next;
        }
        Ok(())
    }

    pub fn write_file(&mut self, path: &str, content: &[u8]) -> Result<(), VfsError> {
        let normalized = normalize_path(path)?;
        let parent = parent_path(&normalized).ok_or_else(|| VfsError::InvalidPath(path.into()))?;
        if !matches!(self.nodes.get(&parent), Some(VfsNode::Directory { .. })) {
            return Err(VfsError::NotDirectory(parent));
        }
        let current_size = self.used_bytes();
        let previous_size = match self.nodes.get(&normalized) {
            Some(VfsNode::File { content, .. }) => content.len() as u64,
            Some(VfsNode::Directory { .. }) => return Err(VfsError::IsDirectory(normalized)),
            None => 0,
        };
        let projected = current_size
            .saturating_sub(previous_size)
            .saturating_add(content.len() as u64);
        if projected > self.quota_bytes {
            return Err(VfsError::QuotaExceeded {
                requested_bytes: projected,
                quota_bytes: self.quota_bytes,
            });
        }
        if !self.nodes.contains_key(&normalized) {
            let name = normalized.rsplit('/').next().unwrap_or_default();
            self.insert_child(&parent, name)?;
        }
        self.nodes.insert(normalized, VfsNode::File { content: content.to_vec(), mode: 0o644 });
        Ok(())
    }

    pub fn append_file(&mut self, path: &str, content: &[u8]) -> Result<(), VfsError> {
        let mut combined = self.read_file(path).unwrap_or_default();
        combined.extend_from_slice(content);
        self.write_file(path, &combined)
    }

    pub fn read_file(&self, path: &str) -> Result<Vec<u8>, VfsError> {
        let normalized = normalize_path(path)?;
        match self.nodes.get(&normalized) {
            Some(VfsNode::File { content, .. }) => Ok(content.clone()),
            Some(VfsNode::Directory { .. }) => Err(VfsError::IsDirectory(normalized)),
            None => Err(VfsError::NotFound(normalized)),
        }
    }

    pub fn read_text(&self, path: &str) -> Result<String, VfsError> {
        let bytes = self.read_file(path)?;
        String::from_utf8(bytes).map_err(|_| VfsError::NotUtf8(path.into()))
    }

    pub fn list_dir(&self, path: &str) -> Result<Vec<String>, VfsError> {
        let normalized = normalize_path(path)?;
        match self.nodes.get(&normalized) {
            Some(VfsNode::Directory { children }) => Ok(children.iter().cloned().collect()),
            Some(VfsNode::File { .. }) => Err(VfsError::NotDirectory(normalized)),
            None => Err(VfsError::NotFound(normalized)),
        }
    }

    pub fn remove(&mut self, path: &str) -> Result<(), VfsError> {
        let normalized = normalize_path(path)?;
        if normalized == "/" {
            return Err(VfsError::ProtectedPath(normalized));
        }
        if let Some(VfsNode::Directory { children }) = self.nodes.get(&normalized)
            && !children.is_empty()
        {
            return Err(VfsError::DirectoryNotEmpty(normalized));
        }
        if self.nodes.remove(&normalized).is_none() {
            return Err(VfsError::NotFound(normalized));
        }
        if let Some(parent) = parent_path(&normalized)
            && let Some(VfsNode::Directory { children }) = self.nodes.get_mut(&parent)
            && let Some(name) = normalized.rsplit('/').next()
        {
            children.remove(name);
        }
        Ok(())
    }

    #[must_use]
    pub fn exists(&self, path: &str) -> bool {
        normalize_path(path).ok().is_some_and(|path| self.nodes.contains_key(&path))
    }

    #[must_use]
    pub fn used_bytes(&self) -> u64 {
        self.nodes
            .values()
            .map(|node| match node {
                VfsNode::File { content, .. } => content.len() as u64,
                VfsNode::Directory { .. } => 0,
            })
            .sum()
    }

    #[must_use]
    pub fn quota_bytes(&self) -> u64 {
        self.quota_bytes
    }

    pub fn file_sha256(&self, path: &str) -> Result<String, VfsError> {
        let bytes = self.read_file(path)?;
        Ok(hex::encode(Sha256::digest(bytes)))
    }

    fn insert_child(&mut self, parent: &str, child: &str) -> Result<(), VfsError> {
        match self.nodes.get_mut(parent) {
            Some(VfsNode::Directory { children }) => {
                children.insert(child.into());
                Ok(())
            }
            Some(VfsNode::File { .. }) => Err(VfsError::NotDirectory(parent.into())),
            None => Err(VfsError::NotFound(parent.into())),
        }
    }
}

pub fn normalize_path(path: &str) -> Result<String, VfsError> {
    if path.contains('\0') {
        return Err(VfsError::InvalidPath(path.into()));
    }
    let absolute = if path.starts_with('/') { path.to_string() } else { format!("/{path}") };
    let mut segments = Vec::new();
    for segment in absolute.split('/') {
        match segment {
            "" | "." => {}
            ".." => {
                if segments.pop().is_none() {
                    return Err(VfsError::TraversalDenied(path.into()));
                }
            }
            value => segments.push(value),
        }
    }
    Ok(if segments.is_empty() { "/".into() } else { format!("/{}", segments.join("/")) })
}

fn parent_path(path: &str) -> Option<String> {
    if path == "/" {
        return None;
    }
    let (parent, _) = path.rsplit_once('/')?;
    Some(if parent.is_empty() { "/".into() } else { parent.into() })
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum VfsError {
    #[error("invalid virtual path: {0}")]
    InvalidPath(String),
    #[error("path traversal denied: {0}")]
    TraversalDenied(String),
    #[error("virtual path not found: {0}")]
    NotFound(String),
    #[error("not a virtual directory: {0}")]
    NotDirectory(String),
    #[error("virtual path is a directory: {0}")]
    IsDirectory(String),
    #[error("virtual directory is not empty: {0}")]
    DirectoryNotEmpty(String),
    #[error("protected virtual path: {0}")]
    ProtectedPath(String),
    #[error("file is not UTF-8: {0}")]
    NotUtf8(String),
    #[error("virtual quota exceeded: requested {requested_bytes}, quota {quota_bytes}")]
    QuotaExceeded { requested_bytes: u64, quota_bytes: u64 },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn traversal_cannot_escape_root() {
        assert_eq!(normalize_path("../../etc/passwd"), Err(VfsError::TraversalDenied("../../etc/passwd".into())));
    }

    #[test]
    fn default_filesystem_contains_training_script() {
        let fs = VirtualFileSystem::dgx_default();
        assert!(fs.read_text("/home/learner/train.sbatch").unwrap().contains("#SBATCH"));
    }

    #[test]
    fn quota_is_enforced() {
        let mut fs = VirtualFileSystem::new(4);
        fs.mkdir_all("/home").unwrap();
        assert!(matches!(fs.write_file("/home/x", b"12345"), Err(VfsError::QuotaExceeded { .. })));
    }
}
