use camino::Utf8PathBuf;

use crate::error::*;
use crate::path::RootPath;
use rayon::prelude::*;
use std::time::Instant;

use snafu::prelude::*;
use tiny_skia::Pixmap;
use typst::{
    diag::Warned,
    foundations::eco_format,
    syntax::{FileId, Source, VirtualPath},
};
use typst_cli::args::{DiagnosticFormat, FontArgs, Input, PackageArgs, ProcessArgs, WorldArgs};
use typst_cli::compile::print_diagnostics;
use typst_cli::world::SystemWorld;
use typst_library::layout::PagedDocument;
use typst_pdf::PdfOptions;

use crate::{frontmatter::split_frontmatter, markdown_it::markdown_to_typst_content, theme::Theme};

#[derive(Clone, Debug)]
pub struct CompiledZine {
    source: RootPath,
    inner: PagedDocument,
}

impl CompiledZine {
    pub fn to_pdf(&self) -> Result<(), Error> {
        let now = Instant::now();

        let mut out = self.source.absolute();
        out.set_extension("pdf");

        // TODO: error
        let pdf_bytes = typst_pdf::pdf(&self.inner, &PdfOptions::default()).unwrap();
        debug!("PDF export: {:.2?}", now.elapsed());

        let now = Instant::now();
        std::fs::write(&out, &pdf_bytes).context(PDFWriteSnafu {
            path: out.to_path_buf(),
        })?;
        debug!("PDF write: {:.2?}", now.elapsed());

        Ok(())
    }

    pub fn to_pixmap(&self) -> Vec<(usize, Pixmap)> {
        let now = Instant::now();

        let res: Vec<(usize, Pixmap)> = self
            .inner
            .pages
            .par_iter()
            .map(|p| (p.number, typst_render::render(&p, 90.0)))
            .collect();

        debug!("PIXMAP export: {:.2?}s", now.elapsed());

        res
    }

    pub fn to_png(&self) -> Result<(), Error> {
        let now = Instant::now();

        self.to_pixmap().par_iter().try_for_each(|(k, v)| {
            let png = v.encode_png().unwrap();
            let mut out = self.source.absolute();
            out.set_extension(&format!("{k}.png"));
            std::fs::write(&out, &png).context(PDFWriteSnafu {
                path: out.to_path_buf(),
            })?;
            Ok(())
        })?;

        debug!("PNG write: {:.2?}s", now.elapsed());
        Ok(())
    }
}

/// A zine on disk inside the [`BaseDir`].
///
/// Create with [`TypstEnv::load`], then compile with [`ZineFile::compile`].
#[derive(Clone, Debug)]
pub struct ZineFile {
    pub file: RootPath,
    // This is the main document in the typst environment, so let's say it's the zine we're compiling???
    #[allow(dead_code)]
    pub source: Source,
}

impl ZineFile {
    pub fn new(path: &RootPath) -> Self {
        Self {
            file: path.clone(),
            // TODO: async/error
            source: Source::new(
                FileId::new_fake(VirtualPath::new(path.path.as_std_path())),
                std::fs::read_to_string(&path.absolute()).unwrap(),
            ),
        }
    }

    pub fn compile(&self) -> Result<CompiledZine, Error> {
        let now = Instant::now();

        let input = Input::Path(self.file.absolute().into());

        let world_args = WorldArgs {
            root: Some(self.file.root.as_std_path().to_path_buf()),
            inputs: Vec::new(),
            font: FontArgs {
                font_paths: Vec::new(),
                ignore_system_fonts: false,
            },
            package: PackageArgs {
                package_path: None,
                package_cache_path: None,
            },
            creation_timestamp: None,
        };

        let process_args = ProcessArgs {
            jobs: None,
            diagnostic_format: DiagnosticFormat::Human,
            features: Vec::new(),
        };

        let world = SystemWorld::new(&input, &world_args, &process_args).unwrap();
        let Warned { output, warnings } = typst::compile::<PagedDocument>(&world);

        let Ok(output) = output else {
            print_diagnostics(
                &world,
                &output.unwrap_err(),
                &warnings,
                DiagnosticFormat::Human,
            )
            .map_err(|err| eco_format!("failed to print diagnostics ({err})"))
            .unwrap();
            error!("FAILED TO COMPILE ZINE.");
            return Err(Error::Typst {
                path: self.file.absolute(),
            });
        };

        for w in &warnings {
            warn!("{:?}", w);
        }

        debug!("Compilation: {:.2?}s", now.elapsed());

        Ok(CompiledZine {
            source: self.file.clone(),
            inner: output,
        })
    }

    pub fn compile_md(&self) -> Result<CompiledZine, Error> {
        let (frontmatter, markdown) = split_frontmatter(&self.file.absolute());

        // Compile once for each theme
        // for (theme_name, theme_settings) in &frontmatter.themes {
        let (theme_name, _theme_settings) = frontmatter.themes.iter().next().unwrap();
        let theme = Theme::new(&self.file.root, theme_name);

        let mut out = String::new();
        out.push_str(&frontmatter.with_typst_header(self, &theme));
        out.push_str(&markdown_to_typst_content(&markdown));

        let mut typst_file = self.file.absolute().to_path_buf();
        typst_file.set_extension(&format!("{theme_name}.typ"));
        std::fs::write(&typst_file, out).context(MDSaveSnafu {
            path: typst_file.to_path_buf(),
        })?;

        info!("Wrote to {typst_file}");

        // Now generate new zine with typ file
        let mut zine = self.clone();
        zine.file = zine.file.root.join(&typst_file);

        zine.compile()
    }

    pub fn relative_dir(&self) -> RootPath {
        let mut zine_dir = self.file.clone();
        zine_dir.path = zine_dir.path.parent().unwrap().to_path_buf();
        zine_dir
    }

    pub fn relative_file(&self) -> RootPath {
        self.file.clone()
    }

    pub fn zine_resource_relative_from_basedir(&self, path: &str) -> Utf8PathBuf {
        let res = self.file.sibling(path).path;
        debug!(
            "zine_resource_relative_from_basedir(basedir: {}, path: {})\n  -> {}",
            self.file.root, path, res
        );
        res
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_name() {
        let zine = ZineFile::from_absolute("/".into(), "/content/a/a.md".into());
        assert_eq!(
            zine.relative_file().relative(),
            Utf8PathBuf::from("content/a/a.md")
        );
        assert_eq!(
            zine.zine_resource_relative_from_basedir("c.png"),
            Utf8PathBuf::from("content/a/c.png")
        );
    }

    #[test]
    fn different_name() {
        let zine = ZineFile::from_absolute("/".into(), "/content/a/b.md".into());
        assert_eq!(
            zine.relative_file().relative(),
            Utf8PathBuf::from("content/a/b.md")
        );
        assert_eq!(
            zine.zine_resource_relative_from_basedir("c.png"),
            Utf8PathBuf::from("content/a/c.png")
        );
    }
}
