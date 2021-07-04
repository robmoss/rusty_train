//! Support writing example output files to specific directories.

#![allow(dead_code)]

use std::path::{Path, PathBuf};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Dir {
    /// The root directory of the accompanying book.
    BookRoot,
    /// The root directory of the developer guide.
    DevGuide,
    /// The root directory of the user guide.
    UserGuide,
    /// The directory for outputs that are not included in the book or guides.
    Examples,
    /// The working directory (the repository root).
    Root,
}

impl Dir {
    /// Return the root directory.
    pub fn root(&self) -> &'static str {
        use Dir::*;
        match self {
            Root => ".",
            Examples => "./examples/output",
            BookRoot => "./book/src",
            DevGuide => "./book/src/dev_guide",
            UserGuide => "./book/src/user_guide",
        }
    }

    /// Return the full path to an output file.
    pub fn join<P: AsRef<Path>>(&self, filename: P) -> PathBuf {
        Path::new(self.root()).join(filename)
    }
}

impl From<&Dir> for &'static Path {
    fn from(src: &Dir) -> &'static Path {
        Path::new(src.root())
    }
}

impl From<Dir> for &'static Path {
    fn from(src: Dir) -> &'static Path {
        Path::new(src.root())
    }
}
