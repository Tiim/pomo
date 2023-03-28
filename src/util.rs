use chrono::{DateTime, Local, LocalResult, NaiveDateTime, NaiveTime, TimeZone, Utc};

#[derive(Debug)]
pub enum FixMeLaterError {
    S(String),
}

pub fn parse_time_string(s: &str) -> Result<DateTime<Utc>, FixMeLaterError> {
    let time;
    match NaiveTime::parse_from_str(s, "%H:%M") {
        Err(e) => return Err(FixMeLaterError::S(e.to_string())),
        Ok(d) => time = d,
    }
    let date_time = NaiveDateTime::new(Utc::now().date_naive(), time);
    match Local.from_local_datetime(&date_time) {
        LocalResult::None => Err(FixMeLaterError::S("Could not find datetime".to_string())),
        LocalResult::Single(s) => Ok(s.with_timezone(&Utc)),
        LocalResult::Ambiguous(_, _) => {
            Err(FixMeLaterError::S("No unambiguous datetime".to_string()))
        }
    }
}
