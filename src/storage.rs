
use notify::{RecursiveMode, Event, Config, RecommendedWatcher, Watcher};

use crate::FixMeLaterError;
use crate::pomo::Pomodoro;
use std::fs;
use std::fs::File;
use std::io::ErrorKind;

use std::path::Path;
use std::sync::mpsc::Receiver;

const CURRENT_FILE: &str = "~/.local/state/pomocl/current_pomo";
//const HISTORY_FILE: &str = "~/.local/state/pomocl/history";

pub fn current_pomo() -> Result<Pomodoro, FixMeLaterError> {
    let file = open_file(CURRENT_FILE, FileMode::Read)?;
    let pomo: Pomodoro = serde_json::from_reader(&file)?;
    Ok(pomo)
}

pub fn write_current_pomo(pomo: Pomodoro) -> Result<(), FixMeLaterError> {
    let file = open_file(CURRENT_FILE, FileMode::Write)?;
    serde_json::to_writer_pretty(&file, &pomo)?;
    Ok(())
}

pub fn subscribe_current_pomo() -> Result<(Receiver<Result<Event, notify::Error>>, RecommendedWatcher), FixMeLaterError> {
    let (tx, rx) = std::sync::mpsc::channel();

    let mut watcher = match notify::RecommendedWatcher::new(tx, Config::default()) {
        Ok(w) => w,
        Err(err) => return Err(FixMeLaterError::S(format!("Error when subscribing to pomo file: {:?}", err))),
    };

    let folder = shellexpand::tilde(CURRENT_FILE);
    match watcher.watch(Path::new(&folder.to_string()), RecursiveMode::NonRecursive) {
        Ok(_) => (),
        Err(err) => return Err(FixMeLaterError::S(format!("{}", err))),
    }

    Ok((rx, watcher))
}

enum FileMode {
    Read,
    Write,
}

fn open_file(file: &str, mode: FileMode) -> Result<File, FixMeLaterError> {
    let folder = shellexpand::tilde(Path::new(file).parent().unwrap().to_str().unwrap()).to_string();
    let file = shellexpand::tilde(file).to_string();

    if let Err(err) = fs::create_dir_all(&folder) {
        if err.kind() != ErrorKind::AlreadyExists {
            return Err(FixMeLaterError::S(format!(
                "Error creating directory {}: {:?}",
                folder, err
            )));
        }
    }
    let f = match mode {
        FileMode::Read => File::open(&file),
        FileMode::Write => File::create(&file),
    };
    match f {
        Ok(f) => Ok(f),
        Err(e) => Err(FixMeLaterError::S(format!(
            "Can't create file {}: {}",
            file, e
        ))),
    }
}
