mod pomo;
mod storage;
mod util;

use crate::util::FixMeLaterError;
use crate::{pomo::PomodoroSetting, storage::write_current_pomo};
use chrono::Utc;
use pomo::{CurrentSection, PomodoroState};

use core::time;
use std::fs::File;
use std::io::{stdout, Seek, SeekFrom, Write};
use std::process::Command;
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
        "stop" => stop_cmd(),
        "pause" => pause_cmd(),
        "unpause" | "continue" => unpause_cmd(),
        "info" => info_cmd(),
        _ => Err(FixMeLaterError::S(format!("Unknown command {}", args[1]))),
    };

    if let Err(FixMeLaterError::S(str)) = res {
        println!("Cought error: {}", str);
    }
}

fn info_cmd() -> CmdResult {
    let pomo = current_pomo()?;
    if !pomo.active {
        println!("inactive");
        return Ok(());
    }
    if let Some(pause) = pomo.pause_started {
        println!("paused at {}", pause);
    }
    let mut start = pomo.start;
    let now = Utc::now();
    for (i, sec) in pomo.sections.iter().enumerate() {
        let current = if let CurrentSection::Section(cur) = pomo.current_section(now) {
            if i == cur {
                "(Current)"
            } else {
                ""
            }
        } else {
            ""
        };
        println!(
            "{}{} -- from {} until {}",
            current,
            sec.state,
            start,
            start + sec.duration
        );
        start += sec.duration;
    }

    return Ok(());
}

fn pause_cmd() -> CmdResult {
    let mut pomo = current_pomo()?;
    pomo.set_pause(Utc::now());
    write_current_pomo(pomo)?;
    return Ok(());
}

fn unpause_cmd() -> CmdResult {
    let mut pomo = current_pomo()?;
    pomo.set_unpause(Utc::now());
    write_current_pomo(pomo)?;
    return Ok(());
}

fn stop_cmd() -> CmdResult {
    let mut pomo = current_pomo()?;
    pomo.set_active(false);
    write_current_pomo(pomo)?;
    return Ok(());
}

fn status_cmd() -> CmdResult {
    let pomo = current_pomo()?;
    println!("{}", pomo.state(Utc::now()));

    return Ok(());
}

fn start_cmd(args: Vec<String>) -> CmdResult {
    let pomodoro_string = if let Some(pstring) = args.get(0) {
        pstring
    } else {
        ""
    };

    let pomo_settings = PomodoroSetting::from_string(pomodoro_string, Utc::now());
    let pomo = pomo_settings.to_pomodoro();

    println!("{}", pomo.state(Utc::now()));

    write_current_pomo(pomo)?;
    return Ok(());
}

fn watch_cmd(args: Vec<String>) -> CmdResult {
    let mut f = args.get(0).map(|path| File::create(path).unwrap());

    let pomodoro = current_pomo()?;

    let mut pomodoro_state = PomodoroState::NotStarted;

    loop {
        let cur_state = pomodoro.state(Utc::now());
        if cur_state.current_state != pomodoro_state {
            pomodoro_state = cur_state.current_state;
            Command::new("notify-send")
                .arg(format!("Pomodoro State {}!", pomodoro_state))
                .output()
                .unwrap();
        }
        let state = pomodoro.state(Utc::now());
        if let Some(ref mut file) = f {
            file.set_len(0)?;
            file.seek(SeekFrom::Start(0))?;
            file.write_all(format!("{}", state).as_bytes())?;
        }
        print!("\r{}        ", state);
        stdout().flush().unwrap();
        thread::sleep(time::Duration::from_secs(1));
    }
}

fn print_help() {
    println!("pomo start [4][p45][b15]");
    println!("  starts the pomodoro timer in this case with");
    println!("  4 times 45min of work and 15min ob breaks.");
    println!("pomo watch [outfile]");
    println!("  prints the current state every second.");
    println!("  if outfile is given, it will be written to every second");
}

impl From<std::io::Error> for FixMeLaterError {
    fn from(value: std::io::Error) -> Self {
        FixMeLaterError::S(format!("{:?}", value))
    }
}

impl From<serde_json::Error> for FixMeLaterError {
    fn from(value: serde_json::Error) -> Self {
        FixMeLaterError::S(format!("{:?}", value))
    }
}
