use chrono::{DateTime, Duration, Utc};
use core::fmt::Display;
use lazy_static::lazy_static;
use regex::Regex;
use std::cmp;

pub struct Pomodoro {
    start: DateTime<Utc>,
    repetitions: u32,
    work_time: u32,
    break_time: u32,
}

pub struct CurrentPomoState {
    current_state: PomodoroState,
    next_state: PomodoroState,
    duration: Duration,
    completed: u32,
}

pub enum PomodoroState {
    NotStarted,
    Work,
    Break,
    Done,
}

impl ToString for Pomodoro {
    fn to_string(&self) -> String {
        let state = self.state();
        return format!("{} {} (next: {}) {}/{}", state.current_state, format_duration(state.duration), state.next_state, state.completed, self.repetitions);
    }
}

fn format_duration(d: Duration) -> String{
  format!("{:02}:{:02}:{:02}", d.num_hours(), d.num_minutes() % 60, d.num_seconds() % 60)
}

impl Display for PomodoroState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Self::NotStarted => "not started",
            Self::Work => "work",
            Self::Break => "break",
            Self::Done => "done",
        };

        write!(f, "{}", str)
    }
}

impl Pomodoro {
    /// Parses a string in the format "4p45b15" into the Pomodoro
    /// repetitions: 4, work_time: 45min, break_time: 15min
    pub fn from_string(s: &str, start: DateTime<Utc>) -> Pomodoro {
        lazy_static! {
            static ref REPETITIONS_REGEX: Regex = Regex::new(r"^(\d+)").unwrap();
            static ref WORK_TIME_REGEX: Regex = Regex::new(r"p(\d+)").unwrap();
            static ref BREAK_TIME_REGEX: Regex = Regex::new(r"b(\d+)$").unwrap();
        }
        let repetitions = if let Some(c) = REPETITIONS_REGEX.captures(s) {
            c.get(1).unwrap().as_str().parse().unwrap()
        } else {
            4
        };
        let work_time = if let Some(c) = WORK_TIME_REGEX.captures(s) {
            c.get(1).unwrap().as_str().parse().unwrap()
        } else {
            45
        };
        let break_time = if let Some(c) = BREAK_TIME_REGEX.captures(s) {
            c.get(1).unwrap().as_str().parse().unwrap()
        } else {
            15
        };

        return Pomodoro {
            start,
            repetitions,
            work_time,
            break_time,
        };
    }

    fn total_work_duration(&self) -> Duration {
        Duration::minutes(i64::from(self.work_time))
    }
    fn total_break_duration(&self) -> Duration {
        Duration::minutes(i64::from(self.break_time))
    }
    fn end(&self) -> DateTime<Utc> {
        let work_duration = self.repetitions * self.total_work_duration().to_std().unwrap();
        let break_duration =
            (cmp::max(self.repetitions, 1) - 1) * self.total_break_duration().to_std().unwrap();
        self.start + Duration::from_std(work_duration + break_duration).unwrap()
    }
    pub fn file_format(&self) -> String {
        format!(
            "{} {}p{}b{}\n",
            self.start.to_rfc3339(),
            self.repetitions,
            self.work_time,
            self.break_time,
        )
    }
    pub fn state(&self) -> CurrentPomoState {
        let now = Utc::now();
        let work = self.total_work_duration();
        let br = self.total_break_duration();
        if self.start > now {
            return CurrentPomoState{
                current_state: PomodoroState::NotStarted,
                next_state: PomodoroState::Work,
                duration: self.start - now,
                completed: 0,
            };
        };

        let mut start = self.start;
        let end = self.end();

        for i in 1..=self.repetitions {
            start += work;
            if start >= now {
                let next_state = if start >= end {
                    PomodoroState::Done
                } else {
                    PomodoroState::Break
                };
                return CurrentPomoState{
                    current_state: PomodoroState::Work, 
                    next_state,
                    duration: start - now,
                    completed: i
                };
            }
            start += br;
            if start >= now {
                if start >= end {
                    return CurrentPomoState{
                        current_state: PomodoroState::Done, 
                        next_state: PomodoroState::Done,
                        duration: Duration::zero(), 
                        completed: i,
                    };
                } else {
                    return CurrentPomoState{
                        current_state: PomodoroState::Break, 
                        next_state: PomodoroState::Work,
                        duration: start - now, 
                        completed: i,
                    };
                }
            }
        }

        return CurrentPomoState {
            current_state: PomodoroState::Done, 
            duration: Duration::zero(), 
            next_state: PomodoroState::Done,
            completed: self.repetitions,
        };
    }
}
