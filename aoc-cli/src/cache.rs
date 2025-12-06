//! Input cache for storing puzzle inputs locally

use crate::error::CacheError;
use std::fs;
use std::path::PathBuf;

/// File-based cache for puzzle inputs
///
/// Directory structure: `{user_dir}/{year}_day{day:02}.txt`
pub struct InputCache {
    /// Pre-computed user directory: `{base_dir}/{user_id}`
    user_dir: PathBuf,
}

impl InputCache {
    /// Create a new input cache for a specific user
    pub fn new(mut base_dir: PathBuf, user_id: u64) -> Self {
        base_dir.push(user_id.to_string());
        Self { user_dir: base_dir }
    }

    /// Get the cache path for a specific year/day
    pub fn cache_path(&self, year: u16, day: u8) -> PathBuf {
        self.user_dir.join(format!("{}_day{:02}.txt", year, day))
    }

    /// Check if input is cached
    pub fn contains(&self, year: u16, day: u8) -> bool {
        self.cache_path(year, day).exists()
    }

    /// Get cached input or None if not cached
    pub fn get(&self, year: u16, day: u8) -> Result<Option<String>, CacheError> {
        let path = self.cache_path(year, day);
        if path.exists() {
            let content = fs::read_to_string(&path)?;
            Ok(Some(content))
        } else {
            Ok(None)
        }
    }

    /// Store input in cache
    pub fn put(&self, year: u16, day: u8, input: &str) -> Result<(), CacheError> {
        let path = self.cache_path(year, day);

        // Create user directory if needed
        fs::create_dir_all(&self.user_dir).map_err(|e| {
            CacheError::DirCreation(format!(
                "Failed to create {}: {}",
                self.user_dir.display(),
                e
            ))
        })?;

        fs::write(&path, input)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_cache_path_format() {
        let temp = TempDir::new().unwrap();
        let cache = InputCache::new(temp.path().to_path_buf(), 12345);

        let path = cache.cache_path(2024, 1);
        assert!(path.to_string_lossy().contains("12345"));
        assert!(path.to_string_lossy().contains("2024_day01.txt"));

        let path = cache.cache_path(2023, 25);
        assert!(path.to_string_lossy().contains("2023_day25.txt"));
    }

    #[test]
    fn test_cache_roundtrip() {
        let temp = TempDir::new().unwrap();
        let cache = InputCache::new(temp.path().to_path_buf(), 12345);

        // Initially not cached
        assert!(!cache.contains(2024, 1));
        assert!(cache.get(2024, 1).unwrap().is_none());

        // Store input
        let input = "test input\nline 2\n";
        cache.put(2024, 1, input).unwrap();

        // Now cached
        assert!(cache.contains(2024, 1));
        assert_eq!(cache.get(2024, 1).unwrap(), Some(input.to_string()));
    }
}
