use camino::{Utf8Path, Utf8PathBuf};
use derive_more::{AsRef, Deref, Display, From, Into};

use crate::{error::*, theme::Theme, zine::ZineFile};

/// A base directory where the zines are stored.
///
/// Contains a `content` and a `themes` directories.
#[derive(AsRef, Clone, Debug, Deref, Display, From, Into)]
#[as_ref(forward)]
#[deref(forward)]
pub struct BaseDir(Utf8PathBuf);

impl BaseDir {
    /// Find a parent [`BaseDir`] from a given path.
    pub fn from_child(orig_path: &Utf8Path) -> Result<Self, Error> {
        let mut path = orig_path;

        while let Some(parent) = path.parent() {
            path = parent;
            trace!("Investigating {path} as basedir...");
            if !path.join("content").is_dir() || !path.join("themes").is_dir() {
                continue;
            }

            return Ok(BaseDir(path.to_path_buf()));
        }

        return Err(Error::NoBaseDir {
            path: orig_path.to_path_buf(),
        });
    }

    /// Append a path to a [`BaseDir`].
    pub fn join(&self, path: impl AsRef<Utf8Path>) -> RootPath {
        let path = path.as_ref();

        RootPath {
            root: self.clone(),
            // Remove the basedir prefix if provided (absolute path)
            path: path.strip_prefix(&self).unwrap_or(path).to_path_buf(),
        }
    }
}

/// A relative path from the [`BaseDir`].
///
/// For example `~/Documents/zinifier/themes/communesbrochures/logo.png` becomes
/// `themes/communesbrochures/logo.png` in this struct.
#[derive(Clone, Debug)]
pub struct RootPath {
    pub root: BaseDir,
    pub path: Utf8PathBuf,
}

impl RootPath {
    /// Deduce a [`RootPath`] including the [`BaseDir`] from a specific path.
    pub fn from_path(path: &Utf8Path) -> Result<RootPath, Error> {
        let basedir = BaseDir::from_child(path)?;
        trace!("Found basedir: {basedir}");
        // Safe unwrap because we just matched the prefix in BaseDir::from_child
        let relative_path = path.strip_prefix(&basedir).unwrap();

        Ok(Self {
            root: basedir,
            path: relative_path.to_path_buf(),
        })
    }

    pub fn absolute(&self) -> Utf8PathBuf {
        self.root.0.join(&self.path)
    }

    pub fn relative_to_theme(&self, theme: &Theme) -> Utf8PathBuf {
        self.relative_to(&theme.relative_file().path)
    }

    pub fn relative_to_zine(&self, zine: &ZineFile) -> Utf8PathBuf {
        self.relative_to(&zine.relative_file().path)
    }

    pub fn relative(&self) -> Utf8PathBuf {
        self.path.to_path_buf()
    }

    pub fn relative_to(&self, orig: &Utf8Path) -> Utf8PathBuf {
        let orig_components: Vec<&str> = orig
            .parent()
            .unwrap()
            .components()
            .map(|x| x.as_str())
            .collect();
        let self_components: Vec<&str> = self
            .path
            .parent()
            .unwrap()
            .components()
            .map(|x| x.as_str())
            .collect();

        let mut res = Utf8PathBuf::new();

        let mut components_counter = 0;
        for component in &orig_components {
            if components_counter < self_components.len()
                && &self_components[components_counter] == component
            {
                // Common part, no need to add ..
                components_counter += 1;
            } else {
                // Not same part, need .. for every component that's left
                let mut other_counter = components_counter;
                while other_counter < orig_components.len() {
                    res = res.join("..");
                    other_counter += 1;
                }
                break;
            }
        }

        for component in &self_components[components_counter..] {
            res = res.join(component);
        }

        res = res.join(self.path.file_name().unwrap());

        res
    }

    pub fn with_extension(&self, ext: &str) -> RootPath {
        let mut new = self.clone();
        new.path.set_extension(ext);
        new
    }

    pub fn sibling(&self, sibling: &str) -> RootPath {
        let mut new = self.clone();
        new.path = new.path.parent().unwrap_or("".into()).join(sibling);
        new
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_theme() {
        let theme = Theme::new("/root".into(), "footheme");

        assert_eq!(
            RootPath::new("/root".into(), "content/a/c/d.png".into()).relative_to_theme(&theme),
            Utf8PathBuf::from("../../content/a/c/d.png"),
        );
        assert_eq!(
            RootPath::new("/root".into(), "content/a/c.png".into()).relative_to_theme(&theme),
            Utf8PathBuf::from("../../content/a/c.png"),
        );
        assert_eq!(
            RootPath::new("/root".into(), "content/c.png".into()).relative_to_theme(&theme),
            Utf8PathBuf::from("../../content/c.png"),
        );
        assert_eq!(
            RootPath::new("/root".into(), "c.png".into()).relative_to_theme(&theme),
            Utf8PathBuf::from("../../c.png"),
        );
        assert_eq!(
            RootPath::new("/root".into(), "themes/a/c.png".into()).relative_to_theme(&theme),
            Utf8PathBuf::from("../a/c.png"),
        );
        assert_eq!(
            RootPath::new("/root".into(), "themes/footheme/c.png".into()).relative_to_theme(&theme),
            Utf8PathBuf::from("c.png"),
        );
    }

    #[test]
    fn to_zine() {
        let zine = ZineFile::from_relative("/root".into(), "content/a/b.md".into());

        assert_eq!(
            RootPath::new("/root".into(), "content/a/c/d.png".into()).relative_to_zine(&zine),
            Utf8PathBuf::from("c/d.png"),
        );
        assert_eq!(
            RootPath::new("/root".into(), "content/a/c.png".into()).relative_to_zine(&zine),
            Utf8PathBuf::from("c.png"),
        );
        assert_eq!(
            RootPath::new("/root".into(), "content/c.png".into()).relative_to_zine(&zine),
            Utf8PathBuf::from("../c.png"),
        );
        assert_eq!(
            RootPath::new("/root".into(), "c.png".into()).relative_to_zine(&zine),
            Utf8PathBuf::from("../../c.png"),
        );
        assert_eq!(
            RootPath::new("/root".into(), "themes/a/c.png".into()).relative_to_zine(&zine),
            Utf8PathBuf::from("../../themes/a/c.png"),
        );

        let zine = ZineFile::from_relative("/root".into(), "content/a/b/c.md".into());

        assert_eq!(
            RootPath::new("/root".into(), "content/a/b/d/e.png".into()).relative_to_zine(&zine),
            Utf8PathBuf::from("d/e.png"),
        );
        assert_eq!(
            RootPath::new("/root".into(), "content/a/c.png".into()).relative_to_zine(&zine),
            Utf8PathBuf::from("../c.png"),
        );
        assert_eq!(
            RootPath::new("/root".into(), "content/c.png".into()).relative_to_zine(&zine),
            Utf8PathBuf::from("../../c.png"),
        );
        assert_eq!(
            RootPath::new("/root".into(), "c.png".into()).relative_to_zine(&zine),
            Utf8PathBuf::from("../../../c.png"),
        );
        assert_eq!(
            RootPath::new("/root".into(), "themes/a/c.png".into()).relative_to_zine(&zine),
            Utf8PathBuf::from("../../../themes/a/c.png"),
        );
    }

    #[test]
    fn to() {
        let path = RootPath::new("/root".into(), "content/a/b.md".into());

        assert_eq!(
            path.relative_to(&RootPath::new("/root".into(), "content/a/c.md".into())),
            Utf8PathBuf::from("b.md"),
        );

        assert_eq!(
            path.relative_to(&RootPath::new("/root".into(), "root_file.toml".into())),
            Utf8PathBuf::from("content/a/b.md"),
        );

        assert_eq!(
            path.relative_to(&RootPath::new("/root".into(), "themes/foo/bar.typ".into())),
            Utf8PathBuf::from("../../content/a/b.md"),
        );
    }
}
