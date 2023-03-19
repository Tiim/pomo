use chrono::serde::ts_seconds;
use chrono::{DateTime, Duration, Utc};
use core::fmt::Display;
use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

pub struct PomodoroSetting {
    start: DateTime<Utc>,
    repetitions: u32,
    work_time: u32,
    break_time: u32,
}

#[derive(Serialize, Deserialize)]
pub struct Pomodoro {
    sections: Vec<PomodoroSection>,
    #[serde(with = "ts_seconds")]
    start: DateTime<Utc>,
}

#[serde_with::serde_as]
#[derive(Serialize, Deserialize)]
pub struct PomodoroSection {
    #[serde_as(as = "serde_with::DurationSeconds<i64>")]
    duration: Duration,
    state: PomodoroState,
}

pub struct CurrentPomoState {
    pub current_state: PomodoroState,
    pub next_state: PomodoroState,
    pub duration: Duration,
    pub completed_repetitions: u32,
    pub total_repetitions: u32,
}

#[derive(PartialEq, Serialize, Deserialize, Copy, Clone)]
pub enum PomodoroState {
    NotStarted,
    Work,
    Break,
    Done,
}

fn format_duration(d: Duration) -> String {
    format!(
        "{:02}:{:02}:{:02}",
        d.num_hours(),
        d.num_minutes() % 60,
        d.num_seconds() % 60
    )
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
    pub fn repetitions(&self) -> u32 {
        self.sections
            .iter()
            .filter(|s| s.state == PomodoroState::Work)
            .count()
            .try_into()
            .unwrap()
    }
    pub fn state(&self, time: DateTime<Utc>) -> CurrentPomoState {
        let current_time = time;
        let mut start = self.start;
        let mut completed = 0;
        if start > current_time {
            return CurrentPomoState {
                current_state: PomodoroState::NotStarted,
                next_state: self.sections.get(0).map_or(PomodoroState::Done, |s| s.state),
                duration: start - current_time,
                completed_repetitions: 0,
                total_repetitions: self.repetitions(),
            };
        }
        for (i, s) in self.sections.iter().enumerate() {
            if s.state == PomodoroState::Work {
                completed += 1;
            }
            if start < current_time && start + s.duration > current_time {
                return CurrentPomoState {
                    current_state: s.state,
                    next_state: self
                        .sections
                        .get(i + 1)
                        .map_or(PomodoroState::Done, |sec| sec.state),
                    duration: (start + s.duration) - current_time,
                    completed_repetitions: completed,
                    total_repetitions: self.repetitions(),
                };
            }
            start += s.duration;
        }
        return CurrentPomoState {
            current_state: PomodoroState::Done,
            next_state: PomodoroState::Done,
            duration: Duration::zero(),
            completed_repetitions: self.repetitions(),
            total_repetitions: self.repetitions(),
        };
    }
}

impl Display for CurrentPomoState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(
            format!(
                "{} {} (next: {}) {}/{}",
                self.current_state,
                format_duration(self.duration),
                self.next_state,
                self.completed_repetitions,
                self.total_repetitions
            )
            .as_str(),
        )?;
        Ok(())
    }
}

impl PomodoroSetting {
    pub fn to_pomodoro(&self) -> Pomodoro {
        let mut pomo = Pomodoro {
            sections: vec![],
            start: self.start,
        };
        for i in 0..self.repetitions {
            pomo.sections.push(PomodoroSection {
                duration: self.total_work_duration(),
                state: PomodoroState::Work,
            });
            if i < self.repetitions - 1 {
                pomo.sections.push(PomodoroSection {
                    duration: self.total_break_duration(),
                    state: PomodoroState::Break,
                });
            }
        }
        pomo
    }
    /// Parses a string in the format "4p45b15" into the Pomodoro
    /// repetitions: 4, work_time: 45min, break_time: 15min
    pub fn from_string(s: &str, start: DateTime<Utc>) -> PomodoroSetting {
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

        return PomodoroSetting {
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
}
