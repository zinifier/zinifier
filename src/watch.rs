use camino::{Utf8Path, Utf8PathBuf};
use glob::glob;
use tokio::runtime::Builder as RuntimeBuilder;
use watchexec::Watchexec;
use watchexec_signals::Signal;

use std::time::Duration;

use crate::{cli::SourceType, path::RootPath, typ::CompileMode};

fn to_be_watched(
    root: &Utf8Path,
    file: &Utf8Path,
    root_filter: impl Fn(&Utf8Path) -> Option<Utf8PathBuf>,
    parent_filter: impl Fn(&Utf8Path) -> Option<Utf8PathBuf>,
) -> Vec<Utf8PathBuf> {
    // Watch files in the 'file' directory
    let mut watched: Vec<Utf8PathBuf> = glob(&format!("{}/*", file.parent().unwrap()))
        .unwrap()
        .filter_map(|x| {
            let x = x.unwrap();
            let x = Utf8PathBuf::from_path_buf(x).unwrap();
            return parent_filter(&x);
        })
        .collect();

    // Watch files (not folders) in the 'root' directory
    info!("ROOT {root}");
    let watched2: Vec<Utf8PathBuf> = glob(&format!("{}/themes/**/*", root))
        .unwrap()
        .filter_map(|x| {
            // let x = x.unwrap().into_path();
            let x = x.unwrap();
            let x = Utf8PathBuf::from_path_buf(x).unwrap();
            return root_filter(&x);
        })
        .collect();

    watched.extend(watched2);

    watched
}

pub fn watch(sourcetype: &SourceType, path: &RootPath) {
    // We watch a specific file, but in the context of an entire basedir...
    let root = path.root.to_path_buf();
    let file = path.clone();

    let root2 = root.to_path_buf();
    let file2 = file.clone();
    let sourcetype2 = sourcetype.clone();

    // First compile a first time
    let _ = sourcetype.compile(&file2, CompileMode::Pdf);

    let parent_filter = match sourcetype {
        SourceType::Markdown => is_not_pdf_or_typ,
        SourceType::Typst => is_not_pdf,
    };

    let rt = RuntimeBuilder::new_current_thread()
        .enable_time()
        .enable_io()
        .build()
        .unwrap();
    rt.block_on(async {
        info!("Watching {root2}");
        let wx = Watchexec::new(move |mut action| {
            // if Ctrl-C is received, quit
            if action.signals().any(|sig| sig == Signal::Interrupt) {
                action.quit();
                return action;
                // exit(0);
            }

            for event in action.events.iter() {
                trace!("WATCHEXEC EVENT: {event:?}");
            }

            if let Err(e) = sourcetype2.compile(&file2, CompileMode::Pdf) {
                error!("{}", e);
            }

            action
        })
        .unwrap();
        wx.config.pathset(
            to_be_watched(&root, &file.absolute(), is_not_pdf, parent_filter)
                .into_iter()
                .map(|x| x.as_std_path().to_path_buf()),
        );
        wx.config.throttle(Duration::from_millis(100));
        wx.main().await.unwrap().unwrap();
    });
}

fn is_not_pdf(path: &Utf8Path) -> Option<Utf8PathBuf> {
    let name = path.file_name().unwrap();
    if path.is_file() && !name.starts_with(".") && !name.ends_with("pdf") {
        Some(path.to_path_buf())
    } else {
        None
    }
}

fn is_not_pdf_or_typ(path: &Utf8Path) -> Option<Utf8PathBuf> {
    let name = path.file_name().unwrap();
    if !name.starts_with(".") && !name.ends_with("pdf") && !name.ends_with("typ") {
        Some(path.to_path_buf())
    } else {
        None
    }
}
