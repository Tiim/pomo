use chrono::DateTime;
use chrono::Utc;
use lazy_static::lazy_static;
use regex::Regex;

use crate::FixMeLaterError;
use crate::Pomodoro;
use std::fs;
use std::fs::File;
use std::io::ErrorKind;
use std::io::{Read, Write};
use std::path::Path;

const CURRENT_FILE: &str = "~/.local/state/pomocl/current_pomo";
//const HISTORY_FILE: &str = "~/.local/state/pomocl/history";

pub fn current_pomo() -> Result<Pomodoro, FixMeLaterError> {
    let mut file = open_file(CURRENT_FILE, FileMode::Read)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    if let Some(pomo) = parse_pomo_file(content.as_str()).pop() {
        return Ok(pomo);
    }
    return Err(FixMeLaterError::S(
        "no pomodoro loaded from file".to_string(),
    ));
}

pub fn write_current_pomo(pomo: Pomodoro) -> Result<(), FixMeLaterError> {
    let mut file = open_file(CURRENT_FILE, FileMode::Write)?;
    file.write_all(pomo.file_format().as_bytes())?;
    Ok(())
}

enum FileMode {
    Read,
    Write,
}

fn open_file(file: &str, mode: FileMode) -> Result<File, FixMeLaterError> {
    let folder = shellexpand::tilde(Path::new(file).to_str().unwrap()).to_string();
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

fn parse_pomo_file(s: &str) -> Vec<Pomodoro> {
    lazy_static! {
        static ref LINE_REGEX: Regex =
            Regex::new(r"(?m)^(?P<start>\S*)\s+(?P<pomo>\d+(p\d+(b\d+)?)?)$").unwrap();
    }
    LINE_REGEX
        .captures_iter(s)
        .map(|caps| match DateTime::parse_from_rfc3339(&caps["start"]) {
            Ok(start) => {
                let pomo = &caps["pomo"];
                Ok(Pomodoro::from_string(pomo, start.with_timezone(&Utc)))
            }
            Err(_) => Err(""),
        })
        .filter(|p| match p {
            Ok(_) => true,
            Err(_) => false,
        })
        .map(|p| p.unwrap())
        .collect()
}
