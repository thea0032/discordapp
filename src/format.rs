use chrono::{DateTime, Datelike, Local, Timelike};

pub fn format_time(then: DateTime<Local>) -> String {
    let now: DateTime<Local> = Local::now();
    let diff = now.date() - then.date();
    let days = diff.num_days();
    if days > 7 {
        format!("{}/{}/{}", then.month(), then.day(), then.year())
    } else if days == 0 {
        format!("{}:{}", then.hour(), then.minute())
    } else {
        format!("{}:{}-{}", then.hour(), then.minute(), days)
    }
}
