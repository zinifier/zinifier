use camino::{Utf8PathBuf, Utf8Path};

use crate::{
    theme::Theme,
    zine::ZineFile,
};

/// A relative path from the zinifier root.
/// 
/// For example `~/Documents/zinifier/themes/communesbrochures/logo.png` becomes
/// `themes/communesbrochures/logo.png` in this struct.
#[derive(Clone, Debug)]
pub struct RootPath {
    pub root: Utf8PathBuf,
    pub path: Utf8PathBuf,
}

impl RootPath {
    pub fn new(root: &Utf8Path, path: &Utf8Path) -> Self {
        Self {
            root: root.to_path_buf(),
            path: path.to_path_buf(),
        }
    }
    
    pub fn relative_to_theme(&self, theme: &Theme) -> Utf8PathBuf {
        self.relative_to(&theme.relative_file())
    }

    pub fn relative_to_zine(&self, zine: &ZineFile) -> Utf8PathBuf {
        self.relative_to(&zine.relative_file())
    }

    pub fn relative(&self) -> Utf8PathBuf {
        self.path.to_path_buf()
    }

    pub fn relative_to(&self, orig: &RootPath) -> Utf8PathBuf {
        let orig_components: Vec<&str> = orig.path.parent().unwrap().components().map(|x| x.as_str()).collect();
        let self_components: Vec<&str> = self.path.parent().unwrap().components().map(|x| x.as_str()).collect();
        
        let mut res = Utf8PathBuf::new();

        let mut components_counter = 0;
        for component in &orig_components {
            if components_counter < self_components.len() && &self_components[components_counter] == component {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_theme()  {
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
    fn to_zine()  {
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
    fn to()  {
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
