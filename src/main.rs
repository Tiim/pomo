mod pomo;
mod storage;
mod util;

use crate::util::FixMeLaterError;
use crate::{pomo::Pomodoro, storage::write_current_pomo};
use chrono::Utc;

use core::time;
use std::fs::File;
use std::io::{Seek, SeekFrom, Write, stdout};
use std::{env, thread};
use storage::current_pomo;

type CmdResult = Result<(), FixMeLaterError>;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() == 1 {
        println!("not enough arguments");
        print_help();
        return;
    }

    let res = match args[1].as_str() {
        "start" => start_cmd(args.as_slice()[2..].to_vec()),
        "status" => status_cmd(),
        "watch" => watch_cmd(args.as_slice()[2..].to_vec()),
        _ => Err(FixMeLaterError::S(format!("Unknown command {}", args[1]))),
    };

    if let Err(FixMeLaterError::S(str)) = res {
        println!("Cought error: {}", str);
    }
}

fn status_cmd() -> CmdResult {
    let pomo = current_pomo()?;
    println!("{}", pomo.to_string());

    return Ok(());
}

fn start_cmd(args: Vec<String>) -> CmdResult {
    let pomodoro_string = if let Some(pstring) = args.get(0) {
        pstring
    } else {
        ""
    };

    let pomodoro = Pomodoro::from_string(pomodoro_string, Utc::now());
    println!("{}", pomodoro.to_string());
    write_current_pomo(pomodoro)?;
    return Ok(());
}

fn watch_cmd(args: Vec<String>) -> CmdResult {
    let mut f = if let Some(path) = args.get(0) {
        File::create(path)?
    } else {
        File::create("pomodoro.txt")?
    };

    let pomodoro = current_pomo()?;

    loop {
        let pom_str = pomodoro.to_string();
        f.set_len(0)?;
        f.seek(SeekFrom::Start(0))?;
        f.write_all(pom_str.as_bytes())?;
        print!("\r{}        ", pom_str);
        stdout().flush().unwrap();
        thread::sleep(time::Duration::from_secs(1));
    }
}

fn print_help() {
    println!("test help");
}

impl From<std::io::Error> for FixMeLaterError {
    fn from(value: std::io::Error) -> Self {
        FixMeLaterError::S(format!("{:?}", value))
    }
}
