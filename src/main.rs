mod pomo;
mod storage;
mod util;

use crate::util::{FixMeLaterError, parse_time_string};
use crate::{pomo::PomodoroSetting, storage::write_current_pomo};
use chrono::{Utc, NaiveTime, NaiveDateTime, DateTime, Local, TimeZone};
use pomo::{CurrentSection, PomodoroState};

use clap::{command, Arg, ArgMatches, Command};
use core::time;
use std::fs::File;
use std::io::{stdout, Seek, SeekFrom, Write};
use std::process::Command as ProcCommand;
use std::{env, thread};
use storage::current_pomo;
type CmdResult = Result<(), FixMeLaterError>;

fn main() {
    let matches = command!()
        .propagate_version(true)
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("start")
                .arg_required_else_help(false)
                .about("Start a new pomodoro")
                .arg(Arg::new("pom").required(false))
                .arg(
                    Arg::new("until")
                        .short('u')
                        .long("until")
                        .value_name("time")
                        .help(
                            "time in the format HH:MM, adjusts the repetition and work duration to match the provided end time",
                        )
                        .required(false),
                ),
        )
        .subcommand(Command::new("status").about("Prints the current pomo"))
        .subcommand(
            Command::new("watch")
                .about("Watch current pomo and print current state every second")
                .arg_required_else_help(false)
                .arg(
                    Arg::new("file")
                        .required(false)
                        .help("if specified, writes the status text to this file"),
                ),
        )
        .subcommand(Command::new("stop").about("Stops the pomo."))
        .subcommand(Command::new("pause").about("Pauses the pomo, can be resumed with 'unpause'"))
        .subcommand(
            Command::new("unpause")
                .alias("continue")
                .about("Unpauses the pomo"),
        )
        .subcommand(Command::new("info").about("Print list of current pomos"))
        .get_matches();

    let res = match matches.subcommand() {
        Some(("start", sub)) => start_cmd(sub),
        Some(("status", _)) => status_cmd(),
        Some(("watch", sub)) => watch_cmd(sub),
        Some(("stop", _)) => stop_cmd(),
        Some(("pause", _)) => pause_cmd(),
        Some(("unpause", _)) => unpause_cmd(),
        Some(("info", _)) => info_cmd(),
        _ => unreachable!(""),
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

fn start_cmd(args: &ArgMatches) -> CmdResult {
    let s = "".to_string();
    let pomodoro_string = args.get_one::<String>("pom").unwrap_or(&s);
    let until = args.get_one::<String>("until");


    let mut pomo_settings = PomodoroSetting::from_string(pomodoro_string, Utc::now());
    if let Some(until_time) = until {
        let date_time = parse_time_string(until_time)?;
        pomo_settings.adjust_end_to(date_time);
    }
    let pomo = pomo_settings.to_pomodoro();


    println!("{}", pomo.state(Utc::now()));

    write_current_pomo(pomo)?;
    return Ok(());
}

fn watch_cmd(args: &ArgMatches) -> CmdResult {
    let mut f = args
        .get_one::<String>("file")
        .map(|path| File::create(path).unwrap());

    let pomodoro = current_pomo()?;

    let mut pomodoro_state = PomodoroState::NotStarted;

    loop {
        let cur_state = pomodoro.state(Utc::now());
        if cur_state.current_state != pomodoro_state {
            pomodoro_state = cur_state.current_state;
            ProcCommand::new("notify-send")
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
