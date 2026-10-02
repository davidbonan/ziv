use chrono::{Datelike, Local};

use crate::library::domain::import_day::ImportDay;

pub fn today() -> ImportDay {
    let now = Local::now();
    ImportDay {
        year: now.year(),
        month: now.month() as u8,
        day: now.day() as u8,
    }
}
