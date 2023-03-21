use chrono::serde::{ts_seconds, ts_seconds_option};
use chrono::{DateTime, Duration, Utc};
use core::fmt::Display;
use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

pub struct PomodoroSetting {
    start: DateTime<Utc>,
    repetitions: u32,
    work_time: Duration,
    break_time: Duration,
}

#[derive(Serialize, Deserialize)]
pub struct Pomodoro {
    pub sections: Vec<PomodoroSection>,
    #[serde(with = "ts_seconds")]
    pub start: DateTime<Utc>,
    pub active: bool,
    #[serde(with = "ts_seconds_option")]
    pub pause_started: Option<DateTime<Utc>>,
}

#[serde_with::serde_as]
#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct PomodoroSection {
    #[serde_as(as = "serde_with::DurationSeconds<i64>")]
    pub duration: Duration,
    pub state: PomodoroState,
}

pub struct CurrentPomoState {
    pub current_state: PomodoroState,
    pub next_state: PomodoroState,
    pub duration: Duration,
    pub completed_repetitions: u32,
    pub total_repetitions: u32,
    pub pause: bool,
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

pub enum CurrentSection {
    Inactive,
    BeforeStart,
    Section(usize),
    AferEnd,
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
    pub fn end(&self) -> DateTime<Utc> {
        return self.start
            + self
                .sections
                .iter()
                .map(|s| s.duration)
                .reduce(|a, s| a + s)
                .unwrap_or(Duration::zero());
    }
    pub fn current_section(&self, t: DateTime<Utc>) -> CurrentSection {
        if !self.active {
            return CurrentSection::Inactive;
        }
        let time = if let Some(pause_started) = self.pause_started {
            pause_started
        } else {
            t
        };
        let current_time = time;
        let mut start = self.start;
        if start > current_time {
            return CurrentSection::BeforeStart;
        }
        for (i, s) in self.sections.iter().enumerate() {
            if start < current_time && start + s.duration > current_time {
                return CurrentSection::Section(i);
            }
            start += s.duration;
        }
        return CurrentSection::AferEnd;
    }

    pub fn state(&self, t: DateTime<Utc>) -> CurrentPomoState {
        let time = if let Some(pause_started) = self.pause_started {
            pause_started
        } else {
            t
        };
        let pause = self.pause_started.is_some();
        let section = self.current_section(t);
        match section {
            CurrentSection::Inactive => CurrentPomoState {
                current_state: PomodoroState::Done,
                next_state: PomodoroState::Done,
                duration: Duration::zero(),
                completed_repetitions: 0,
                total_repetitions: 0,
                pause,
            },
            CurrentSection::BeforeStart => CurrentPomoState {
                current_state: PomodoroState::NotStarted,
                next_state: self
                    .sections
                    .get(0)
                    .map_or(PomodoroState::Done, |s| s.state),
                duration: self.start - time,
                completed_repetitions: 0,
                total_repetitions: self.repetitions(),
                pause,
            },
            CurrentSection::Section(i) => {
                let current_section = self.sections.get(i).unwrap();
                let start_time = self.start
                    + self
                        .sections
                        .iter()
                        .take(i)
                        .map(|s| s.duration)
                        .reduce(|acc, val| acc + val)
                        .unwrap_or(Duration::zero());
                let next_section = self.sections.get(i + 1);
                let completed = self
                    .sections
                    .iter()
                    .take(i + 1)
                    .filter(|s| s.state == PomodoroState::Work)
                    .count();
                CurrentPomoState {
                    current_state: current_section.state,
                    next_state: next_section.map_or(PomodoroState::Done, |sec| sec.state),
                    duration: (start_time + current_section.duration) - time,
                    completed_repetitions: u32::try_from(completed).unwrap(),
                    total_repetitions: self.repetitions(),
                    pause,
                }
            }
            CurrentSection::AferEnd => CurrentPomoState {
                current_state: PomodoroState::Done,
                next_state: PomodoroState::Done,
                duration: Duration::zero(),
                completed_repetitions: self.repetitions(),
                total_repetitions: self.repetitions(),
                pause,
            },
        }
    }
    pub fn set_active(&mut self, a: bool) {
        self.active = a;
    }
    pub fn set_pause(&mut self, pause_start: DateTime<Utc>) {
        self.pause_started = Some(pause_start);
    }
    pub fn set_unpause(&mut self, pause_end: DateTime<Utc>) {
        if let Some(pause_start) = self.pause_started {
            let sec = self.current_section(pause_start);
            if let CurrentSection::Section(s) = sec {
                let section_start_time = self.start
                    + self
                        .sections
                        .iter()
                        .take(s)
                        .map(|s| s.duration)
                        .reduce(|a, v| a + v)
                        .unwrap_or(Duration::zero());
                let new_section_dur = pause_start - section_start_time;
                assert!(new_section_dur > Duration::zero());
                let split_section_old_dur;
                let split_section_state;
                {
                    let split_section = self.sections.get_mut(s).unwrap();
                    split_section_old_dur = split_section.duration;
                    split_section.duration = new_section_dur;
                    split_section_state = split_section.state;
                }
                self.sections.insert(
                    s + 1,
                    PomodoroSection {
                        duration: pause_end - pause_start,
                        state: PomodoroState::Break,
                    },
                );
                self.sections.insert(
                    s + 2,
                    PomodoroSection {
                        duration: split_section_old_dur - new_section_dur,
                        state: split_section_state,
                    },
                );
            }
            self.pause_started = None;
        }
    }
}

impl Display for CurrentPomoState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let next = if self.next_state != self.current_state {
            format!("(-> {}) ", self.next_state)
        } else {
            "".to_string()
        };
        let duration = if self.current_state != PomodoroState::Done {
            format!("{} ", format_duration(self.duration))
        } else {
            "".to_string()
        };
        let pause = if self.pause { " (paused)" } else { "" };
        f.write_str(
            format!(
                "{} {}{}{}/{}{}",
                self.current_state,
                duration,
                next,
                self.completed_repetitions,
                self.total_repetitions,
                pause,
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
            active: true,
            pause_started: None,
        };
        for i in 0..self.repetitions {
            pomo.sections.push(PomodoroSection {
                duration: self.work_time,
                state: PomodoroState::Work,
            });
            if i < self.repetitions - 1 {
                pomo.sections.push(PomodoroSection {
                    duration: self.break_time,
                    state: PomodoroState::Break,
                });
            }
        }
        pomo
    }
    /// calculate new work time and repetitions based on end time
    pub fn adjust_end_to(&mut self, end_time: DateTime<Utc>) {
        // base formula of total duration, with r = repetitions, w = work time, b = break time:
        // d = rw + (r-1)b
        // rewrite in terms of work time:
        // f(r) = w = (d/r) - ((r-1)b/r)
        assert!(end_time > self.start);
        let d = end_time - self.start;

        let f = |r| (d / r) - (self.break_time * (r - 1)) / r;

        let mut reps = 1;
        let mut w_delta = i64::max_value();
        // loop over repetitions to find the one where the difference between 
        // the calculated and the specified work time is the smallest
        loop {
            let w = f(reps);
            let new_w_delta = i64::abs((w - self.work_time).num_seconds());
            if new_w_delta > w_delta {
                break;
            }
            reps += 1;
            w_delta = new_w_delta;
        }
        reps -= 1;
        let new_w = f(reps);
        self.repetitions = u32::try_from(reps).unwrap();
        self.work_time = new_w;
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
            40
        };
        let break_time = if let Some(c) = BREAK_TIME_REGEX.captures(s) {
            c.get(1).unwrap().as_str().parse().unwrap()
        } else {
            10
        };

        return PomodoroSetting {
            start,
            repetitions,
            work_time: Duration::minutes(work_time),
            break_time: Duration::minutes(break_time),
        };
    }
}
