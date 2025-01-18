use camino::{Utf8PathBuf, Utf8Path};
use snafu::prelude::*;
use typst_cli::args::{DiagnosticFormat, Input, WorldArgs, ProcessArgs, FontArgs, PackageArgs, CompileCommand, CompileArgs, PdfStandard};
use typst_cli::compile::{CompileConfig, compile_once};
use typst_cli::world::SystemWorld;

use std::time::Instant;

use crate::{
    error::*,
    frontmatter::split_frontmatter,
    markdown_it::markdown_to_typst_content,
    theme::Theme,
    zine::ZineFile,
};

pub fn compile_typ(root: &Utf8Path, file: &Utf8Path) -> Result<Utf8PathBuf, Error> {
    info!("Compiling {file} for root {root}");
    let now = Instant::now();

    let input = Input::Path(root.join(file).into());

    let world_args = WorldArgs {
        root: Some(root.into()),
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

    let mut world = SystemWorld::new(&input, &world_args, &process_args).unwrap();

    let compile_args = CompileArgs {
        input: input,
        output: None,
        pages: None,
        make_deps: None,
        format: None,
        open: None,
        ppi: 144.0,
        timings: None,
        pdf_standard: vec!(PdfStandard::V_1_7),
        world: world_args,
        process: process_args,
    };

    let mut compile_config = CompileConfig::new(&CompileCommand { args: compile_args }).unwrap();

    compile_once(&mut world, &mut compile_config).map_err(|e| DummyError::from(e)).context(TypstSnafu { document: file.to_path_buf() });
    let elapsed = now.elapsed();
    println!("Compiled in {:.2?}s", elapsed);

    let mut out_file = file.to_path_buf();
    out_file.set_extension("pdf");

    Ok(out_file)
}

pub fn compile_md(root: &Utf8Path, file: &Utf8Path) -> Result<Utf8PathBuf, MarkdownError> {
    let (frontmatter, markdown) = split_frontmatter(file);

    // let relative_theme = theme.theme_relative_from(file);

    let zine = ZineFile::from_absolute(root, file);

    // Compile once for each theme
    // for (theme_name, theme_settings) in &frontmatter.themes {
    let (theme_name, theme_settings) = frontmatter.themes.iter().next().unwrap();
    let theme = Theme::new(root, theme_name);
    let mut out = String::new();
    out.push_str(&frontmatter.with_typst_header(&zine, &theme));
    out.push_str(&markdown_to_typst_content(&markdown));

    let mut typst_file = file.to_path_buf();
    typst_file.set_extension(&format!("{theme_name}.typ"));
    std::fs::write(&typst_file, out).context(MDSaveSnafu { path: typst_file.to_path_buf() });

    info!("Wrote to {typst_file}");

    return compile_typ(root, &typst_file).context(MDCompileSnafu { path: file.to_path_buf() })
}
