use camino::Utf8PathBuf;
use clap::Parser;

use zinifier::{
    cli::{Action, SourceType},
    path::RootPath,
};

#[derive(Debug, Parser)]
struct Cli {
    action: Action,
    #[clap(short, long, default_value = "pdf")]
    mode: zinifier::typ::CompileMode,
    file: Utf8PathBuf,
}

fn main() {
    if let Err(_) = std::env::var("RUST_LOG") {
        std::env::set_var("RUST_LOG", "info");
    }
    pretty_env_logger::init();
    log::trace!("fun");

    let cli = Cli::parse();

    let absolute_file = cli.file.canonicalize_utf8().unwrap();
    let s = SourceType::from_ext(&absolute_file);
    log::trace!("fun");

    // Deduce BaseDir and relative path
    let file = RootPath::from_path(&absolute_file).unwrap();
    log::trace!("fun");

    let res = match &cli.action {
        Action::Compile => s.compile(&file, cli.mode),
        #[cfg(feature = "watch")]
        Action::Watch => s.watch(&file),
    };

    if let Err(e) = res {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
