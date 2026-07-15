//! A tiny disk cache for fetched documentation pages.
//!
//! Docs.rs and doc.rust-lang.org pages rarely change once a crate version
//! is published, so caching aggressively avoids hammering the network on
//! repeat lookups (and enables `--offline` mode entirely). Keys are the
//! request URL, sanitized into a filesystem-safe name; values are the raw
//! response bodies.

use crate::error::{Result, RsHelpError};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

pub struct Cache {
    dir: PathBuf,
    ttl: Duration,
}

impl Cache {
    /// Locate (and lazily create) the cache directory. Falls back to a
    /// temp-dir subfolder if the platform cache directory is unavailable.
    pub fn new(ttl_secs: u64) -> Self {
        let dir = dirs::cache_dir()
            .unwrap_or_else(std::env::temp_dir)
            .join("rshelp")
            .join("docs");
        Cache {
            dir,
            ttl: Duration::from_secs(ttl_secs),
        }
    }

    fn key_path(&self, key: &str) -> PathBuf {
        let sanitized: String = key
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
            .collect();
        // Guard against pathologically long URLs by truncating the name and
        // relying on collisions being harmless (worst case: a cache miss).
        let truncated: String = sanitized.chars().take(180).collect();
        self.dir.join(format!("{truncated}.cache"))
    }

    /// Return a cached body for `key` if present and not expired (or always,
    /// when `ignore_ttl` is set -- used by `--offline`).
    pub fn get(&self, key: &str, ignore_ttl: bool) -> Option<String> {
        let path = self.key_path(key);
        let meta = fs::metadata(&path).ok()?;
        if !ignore_ttl {
            let modified = meta.modified().ok()?;
            if SystemTime::now().duration_since(modified).unwrap_or_default() > self.ttl {
                return None;
            }
        }
        fs::read_to_string(&path).ok()
    }

    /// Write `body` into the cache under `key`, creating the cache
    /// directory tree as needed.
    pub fn put(&self, key: &str, body: &str) -> Result<()> {
        fs::create_dir_all(&self.dir).map_err(RsHelpError::Io)?;
        let path = self.key_path(key);
        fs::write(path, body).map_err(RsHelpError::Io)?;
        Ok(())
    }

    /// Remove every cached entry.
    pub fn clear(&self) -> Result<usize> {
        if !self.dir.exists() {
            return Ok(0);
        }
        let mut removed = 0usize;
        for entry in fs::read_dir(&self.dir).map_err(RsHelpError::Io)? {
            let entry = entry.map_err(RsHelpError::Io)?;
            if is_cache_file(&entry.path()) {
                fs::remove_file(entry.path()).map_err(RsHelpError::Io)?;
                removed += 1;
            }
        }
        Ok(removed)
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }
}

fn is_cache_file(path: &Path) -> bool {
    path.extension().and_then(|e| e.to_str()) == Some("cache")
}
