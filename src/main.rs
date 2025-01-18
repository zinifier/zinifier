use camino::Utf8PathBuf;
use clap::Parser;

use zinifier::cli::Action;

#[derive(Debug, Parser)]
struct Cli {
    action: Action,
    file: Utf8PathBuf,
}

fn main() {
    if let Err(_) = std::env::var("RUST_LOG") {
        std::env::set_var("RUST_LOG", "info");
    }
    pretty_env_logger::init();

    let cli = Cli::parse();

    let absolute_file = cli.file.canonicalize_utf8().unwrap();
    let project_root = absolute_file.parent().unwrap().parent().unwrap().parent().unwrap();

    cli.action.run(&project_root, &absolute_file);
}
