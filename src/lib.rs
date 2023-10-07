//! # Test Files
//!
//! `test_files` implements some convenient patterns for creating
//! temporary files with given paths (relative to a temporary root)
//! and content.
//!
//! A temporary directory is created on instantiation, and torn
//! down when the returned object falls out of scope.
//!
//! # Example
//!
//! ```
//! use test_files::TestFiles;
//!
//! let temp_dir = TestFiles::new();
//! temp_dir
//!     .file("a/b/c.txt", "ok")
//!     .file("b/c/d.txt", "fine");
//!
//! let file_path = temp_dir.path().join("a").join("b").join("c.txt");
//! let written_content = std::fs::read_to_string(file_path).unwrap();
//! assert_eq!(written_content, "ok");
//!
//! let file_path = temp_dir.path().join("b").join("c").join("d.txt");
//! let written_content = std::fs::read_to_string(file_path).unwrap();
//! assert_eq!(written_content, "fine");
//! ```
//!
//! The pain of creating intermediate directories is abstracted
//! away, so you can just write relative paths, content, and
//! use the created files in tests or otherwise.  The root of
//! the temporary directory is exposed by the `.path()` method.
use std::path::{Path, PathBuf};
use tempfile::{tempdir, TempDir};
use thiserror::Error;
use touch::file;

pub type Result<T, E = TestFilesError> = core::result::Result<T, E>;

#[derive(Error, Debug)]
pub enum TestFilesError {
    #[error("Path error `{path:?}`")]
    PathError { path: String },
    #[error(transparent)]
    FileWriteError(#[from] touch::Error),
    #[error(transparent)]
    TempDirError(#[from] std::io::Error),
}

pub struct TestFiles(TempDir);

impl TestFiles {
    /// Creates a plain file under temporary directory, with specified
    /// content.
    ///
    /// # Examples
    ///
    /// ```
    /// use indoc::indoc;
    /// use std::fs;
    ///
    /// let temp_dir = test_files::TestFiles::new();
    /// temp_dir.file("a/b/c.txt", "ok")
    /// .file("b/c/d.txt", "fine");
    ///
    /// let file_path = temp_dir.path().join("a").join("b").join("c.txt");
    /// let written_content = fs::read_to_string(file_path).unwrap();
    /// assert_eq!(written_content, "ok");
    ///
    /// let file_path = temp_dir.path().join("b").join("c").join("d.txt");
    /// let written_content = fs::read_to_string(file_path).unwrap();
    /// assert_eq!(written_content, "fine");
    /// ```
    pub fn file(&self, path: &str, content: &str) -> &Self {
        self.try_file(path, content).unwrap()
    }

    /// Creates a new temporary directory that is
    /// removed when it goes out of scope.
    ///
    /// Panics on failure
    ///
    /// # Examples
    ///
    /// ```
    /// let temp_dir = test_files::TestFiles::new();
    ///
    /// assert!(temp_dir.path().is_dir());
    /// ```
    pub fn new() -> Self {
        Self::try_new().unwrap()
    }

    /// Returns the path of the underlying temporary directory.
    ///
    /// # Examples
    ///
    /// ```
    /// let temp_dir = test_files::TestFiles::new();
    ///
    /// assert!(temp_dir.path().is_dir());
    /// ```
    pub fn path(&self) -> &Path {
        self.0.path()
    }

    fn slash(&self, relative_path: &str) -> PathBuf {
        self.path().join(relative_path)
    }

    /// Tries to create a plain file under temporary directory
    /// with specified content.
    ///
    /// # Examples
    ///
    /// ```
    /// use indoc::indoc;
    /// use std::fs;
    ///
    /// # fn main() -> test_files::Result<()> {
    /// let temp_dir = test_files::TestFiles::new();
    /// temp_dir.try_file("a/b/c.txt", "ok")?
    /// .try_file("b/c/d.txt", "fine")?;
    ///
    /// let file_path = temp_dir.path().join("a").join("b").join("c.txt");
    /// let written_content = fs::read_to_string(file_path).unwrap();
    /// assert_eq!(written_content, "ok");
    ///
    /// let file_path = temp_dir.path().join("b").join("c").join("d.txt");
    /// let written_content = fs::read_to_string(file_path).unwrap();
    /// assert_eq!(written_content, "fine");
    /// #   Ok(())
    /// # }
    /// ```
    pub fn try_file(&self, path: &str, content: &str) -> Result<&Self> {
        file::write(
            self.slash(path).to_str().ok_or(TestFilesError::PathError {
                path: path.to_string(),
            })?,
            content,
            true,
        )?;
        Ok(self)
    }

    /// Tries to create a new temporary directory that is
    /// removed when it goes out of scope.
    ///
    /// # Examples
    ///
    /// ```
    /// let temp_dir = test_files::TestFiles::try_new();
    ///
    /// assert!(temp_dir.is_ok());
    /// assert!(temp_dir.unwrap().path().is_dir());
    /// ```
    pub fn try_new() -> Result<Self> {
        Ok(Self(tempdir()?))
    }
}

impl Default for TestFiles {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use color_eyre::Result;
    use indoc::indoc;
    use std::fs;

    #[test]
    fn makes_deletes_files() -> Result<()> {
        let tmp_path: Option<PathBuf>;
        {
            let files = TestFiles::new();
            tmp_path = Some(files.path().to_owned());

            let content = indoc! {"
                ---
                version: 3
            "};

            files.file("a/b/index.yml", content);
            let file_path = tmp_path
                .as_ref()
                .unwrap()
                .join("a")
                .join("b")
                .join("index.yml");
            let written_content = fs::read_to_string(file_path).unwrap();
            assert_eq!(written_content, content);
        }
        // directory is cleaned up with TestFiles falls out of scope
        assert!(!tmp_path.unwrap().is_dir());
        Ok(())
    }
}
