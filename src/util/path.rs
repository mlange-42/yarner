//! Path and file utilities.

use std::path::PathBuf;

/// Path and file utilities.
#[allow(dead_code)]
pub struct PathUtil {}

#[allow(dead_code)]
impl PathUtil {
    /// Get the file extension from a path.
    pub fn extension(path: &PathBuf) -> Option<String> {
        path.extension()
            .and_then(|ext| ext.to_str().map(|ext| ext.to_lowercase()))
    }
    /// Get the file's base name from a path (name without extension).
    pub fn stem(path: &PathBuf) -> Option<String> {
        path.file_stem()
            .and_then(|st| st.to_str().map(|st| st.to_string()))
    }
    /// Get the file's name from a path (with extension).
    pub fn name(path: &PathBuf) -> Option<String> {
        path.file_name()
            .and_then(|st| st.to_str().map(|st| st.to_string()))
    }
    /// Get the file's base name from a path (name without extension).
    pub fn out_path(in_path: &PathBuf, out_pattern: &str) -> Option<PathBuf> {
        let name = PathUtil::stem(in_path);
        match name {
            Some(name) => Some(PathBuf::from(out_pattern.replace("*", &name))),
            None => None,
        }
    }
    /// List all files for a pattern
    pub fn list_files(pattern: &str) -> Result<Vec<PathBuf>, glob::PatternError> {
        let paths: glob::Paths = glob::glob(pattern)?;
        let vec = paths
            .filter(|p| p.is_ok() && p.as_ref().unwrap().is_file())
            .map(|p| p.unwrap())
            .collect();
        Ok(vec)
    }
    /// List all files for multiple patterns
    pub fn list_all_files(patterns: &[String]) -> Result<Vec<PathBuf>, glob::PatternError> {
        let mut vec = vec![];
        for files in patterns.iter().map(|pat| Self::list_files(pat)) {
            for f in files? {
                vec.push(f);
            }
        }
        Ok(vec)
    }
    /// List all files for multiple patterns
    pub fn list_all_files_str(patterns: &[&str]) -> Result<Vec<PathBuf>, glob::PatternError> {
        let mut vec = vec![];
        for files in patterns.iter().map(|pat| Self::list_files(pat)) {
            for f in files? {
                vec.push(f);
            }
        }
        Ok(vec)
    }
    /// List all directories for a pattern
    pub fn list_dirs(pattern: &str) -> Result<Vec<PathBuf>, glob::PatternError> {
        let paths: glob::Paths = glob::glob(pattern)?;
        let vec = paths
            .filter(|p| p.is_ok() && p.as_ref().unwrap().is_dir())
            .map(|p| p.unwrap())
            .collect();
        Ok(vec)
    }
    /// List all files and directories for a pattern
    pub fn list_all(pattern: &str) -> Result<Vec<PathBuf>, glob::PatternError> {
        let paths: glob::Paths = glob::glob(pattern)?;
        let vec = paths.filter(|p| p.is_ok()).map(|p| p.unwrap()).collect();
        Ok(vec)
    }
}

#[cfg(test)]
mod test {
    use crate::util::PathUtil;
    use std::path::PathBuf;

    #[test]
    fn extension() {
        let path = PathBuf::from("C:/a/b/abc.jpg");
        let ext = PathUtil::extension(&path);

        assert_eq!(ext.unwrap(), "jpg")
    }

    #[test]
    fn list_files() {
        let pattern = "./*";
        let list = PathUtil::list_files(&pattern).unwrap();

        assert!(list.contains(&PathBuf::from("Cargo.toml")));
        assert!(!list.contains(&PathBuf::from("src")));
    }
    #[test]
    fn list_dirs() {
        let pattern = "./*";
        let list = PathUtil::list_dirs(&pattern).unwrap();

        assert!(!list.contains(&PathBuf::from("Cargo.toml")));
        assert!(list.contains(&PathBuf::from("src")));
    }
    #[test]
    fn list_all() {
        let pattern = "./*";
        let list = PathUtil::list_all(&pattern).unwrap();

        assert!(list.contains(&PathBuf::from("Cargo.toml")));
        assert!(list.contains(&PathBuf::from("src")));
    }
}
