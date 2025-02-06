use camino::Utf8PathBuf;

use crate::{
    path::{BaseDir, RootPath},
    zine::ZineFile,
};

#[derive(Clone, Debug)]
pub struct Theme {
    pub basedir: BaseDir,
    pub themefile: RootPath,
    pub name: String,
}

impl Theme {
    pub fn new(basedir: &BaseDir, name: &str) -> Self {
        Self {
            basedir: basedir.clone(),
            themefile: basedir.join(&format!("themes/{name}/theme.typ")),
            name: name.to_string(),
        }
    }

    pub fn relative_file(&self) -> RootPath {
        self.themefile.clone()
    }

    pub fn relative_to_zine(&self, zine: &ZineFile) -> Utf8PathBuf {
        self.themefile.relative_to_zine(zine)
    }

    pub fn theme_resource_relative_from_basedir(&self, path: &str) -> Utf8PathBuf {
        let res = Utf8PathBuf::from(&format!("themes/{}/{}", &self.name, path));
        debug!(
            "theme_resource_relative_from_basedir(basedir: {}, path: {})\n  -> {}",
            self.basedir, path, res
        );
        res
    }

    pub fn zine_resource_relative_from_theme(&self, path: &str, zine: &ZineFile) -> Utf8PathBuf {
        let res = Utf8PathBuf::from("../..").join(zine.zine_resource_relative_from_basedir(path));
        debug!(
            "Resource {} for zine {} is {} from theme",
            path,
            zine.file.absolute(),
            res
        );
        res
    }
}
