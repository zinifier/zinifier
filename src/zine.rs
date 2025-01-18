use camino::{Utf8PathBuf, Utf8Path};

use crate::path::RootPath;

/// A zine that wasn't loaded yet.
pub struct ZineFile {
    #[allow(dead_code)]
    basedir: Utf8PathBuf,
    zinedir: RootPath,
    zinefile: RootPath,
    #[allow(dead_code)]
    name: String,
}

impl ZineFile {
    pub fn from_relative(basedir: &Utf8Path, zine: &Utf8Path) -> Self {
        Self::from_absolute(basedir, &basedir.join(zine))
    }
    
    pub fn from_absolute(basedir: &Utf8Path, zine: &Utf8Path) -> Self {
        let relative_zine = zine.strip_prefix(basedir).unwrap();
        let zinedir = relative_zine.parent().unwrap();
        let name = relative_zine.file_stem().unwrap();

        Self {
            basedir: basedir.to_path_buf(),
            zinedir: RootPath::new(basedir, zinedir),
            zinefile: RootPath::new(basedir, relative_zine),
            name: name.to_string(),
        }
    }

    pub fn relative_dir(&self) -> RootPath {
        self.zinedir.clone()
    }
    
    pub fn relative_file(&self) -> RootPath {
        self.zinefile.clone()
    }

    pub fn zine_resource_relative_from_basedir(&self, path: &str) -> Utf8PathBuf {
        self.zinedir.relative().join(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_name() {
        let zine = ZineFile::from_absolute("/".into(), "/content/a/a.md".into());
        assert_eq!(zine.relative_file().relative(), Utf8PathBuf::from("content/a/a.md"));
        assert_eq!(zine.zine_resource_relative_from_basedir("c.png"), Utf8PathBuf::from("content/a/c.png"));
    }

    #[test]
    fn different_name() {
        let zine = ZineFile::from_absolute("/".into(), "/content/a/b.md".into());
        assert_eq!(zine.relative_file().relative(), Utf8PathBuf::from("content/a/b.md"));
        assert_eq!(zine.zine_resource_relative_from_basedir("c.png"), Utf8PathBuf::from("content/a/c.png"));
    }
}
