use camino::{Utf8PathBuf, Utf8Path};

use crate::{
    path::RootPath,
    zine::ZineFile,
};

#[derive(Clone, Debug)]
pub struct Theme {
    pub basedir: Utf8PathBuf,
    pub themefile: RootPath,
    pub name: String,
}

impl Theme {
    pub fn new(basedir: &Utf8Path, name: &str) -> Self {
        Self {
            basedir: basedir.to_path_buf(),
            themefile: RootPath::new(basedir, &Utf8PathBuf::from(&format!("themes/{name}/theme.typ"))),
            name: name.to_string(),
        }
    }

    pub fn relative_file(&self) -> RootPath {
        self.themefile.clone()
    }

    pub fn relative_to_zine(&self, zine: &ZineFile) -> Utf8PathBuf {
        self.themefile.relative_to_zine(zine)
    }

    // pub fn theme_relative_from_relative(&self, orig: &Utf8Path) -> Utf8PathBuf {
    //     self.themefile.relative_to(&RootPath::new(&self.basedir, orig))
    // }

    // pub fn theme_relative_from_relative(&self, path: &Utf8Path) -> Utf8PathBuf {
    //     // This would be for absolute path
    //     // let relative = path.strip_prefix(&self.basedir).expect(
    //     //     &format!("{path} is not in the basedir {}", &self.basedir)
    //     // );

    //     // So we have the relative path to `path` from the basedir.
    //     // Now we want to do the opposite way and append the theme path
    //     // eg go from content/ZINE/ZINE.md to ../../
    //     let num_climb = path.components().count() - 1;

    //     let theme_path = Utf8PathBuf::from("../".repeat(num_climb));
    //     theme_path.join(&self.theme_relative())
    // }

    pub fn theme_resource_relative_from_basedir(&self, path: &str) -> Utf8PathBuf {
        Utf8PathBuf::from(&format!("themes/{}/{}", &self.name, path))
    }

    pub fn zine_resource_relative_from_theme(&self, path: &str, zine: &ZineFile) -> Utf8PathBuf {
        Utf8PathBuf::from("../..").join(zine.zine_resource_relative_from_basedir(path))
    }
}
