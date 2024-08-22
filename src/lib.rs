//! # Workdays
//!
//! `workdays` is a Rust library for computing work days and handling work calendars.
//! It provides functionality to define custom work weeks, add holidays, and compute
//! dates based on a given number of work days.
//!
//! ## Features
//!
//! - Compute end dates based on work days
//! - Calculate the number of work days between two dates
//! - Parse and handle work calendar configurations (YAML or JSON)
//! - Support for custom work days and holidays
//! - Flexible weekday parsing
//!
//! ## Usage
//!
//! Here's a quick example:
//!
//! ```
//! use workdays::WorkCalendar;
//! use chrono::NaiveDate;
//!
//! let mut calendar = WorkCalendar::new();
//! calendar.add_holiday(NaiveDate::from_ymd_opt(2023, 12, 25).unwrap());
//! calendar.set_work_days("Mon,Tue,Wed,Thu,Fri").unwrap();
//!
//! let start_date = NaiveDate::from_ymd_opt(2023, 8, 21).unwrap();
//! let days_worked = 20;
//!
//! let (end_date, calendar_duration) = calendar.compute_end_date(start_date, days_worked).unwrap();
//! println!("End date: {}", end_date);
//! println!("Calendar duration: {} days", calendar_duration.num_days());
//! ```

use chrono::{Datelike, Duration, NaiveDate, Weekday};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, str::FromStr};

/// Represents a work calendar with customizable work days and holidays.
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct WorkCalendar {
    work_days: HashSet<Weekday>,
    holidays: HashSet<NaiveDate>,
}

impl FromStr for WorkCalendar {
    type Err = Box<dyn std::error::Error>;

    /// Creates a `WorkCalendar` from a YAML or JSON string.
    ///
    /// # Arguments
    ///
    /// * `s` - A string slice that holds the configuration in YAML or JSON format.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::str::FromStr;
    /// use workdays::WorkCalendar;
    ///
    /// let config = r#"
    /// work_days:
    ///   - Monday
    ///   - Tuesday
    ///   - Wednesday
    /// holidays:
    ///   - 2023-12-25
    /// "#;
    ///
    /// let calendar = WorkCalendar::from_str(config).unwrap();
    /// assert!(calendar.is_work_day(&chrono::Weekday::Mon));
    /// assert!(!calendar.is_work_day(&chrono::Weekday::Thu));
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let config: WorkCalendarConfig = if s.trim_start().starts_with('{') {
            serde_json::from_str(s)?
        } else {
            serde_yaml::from_str(s)?
        };
        Ok(Self::from(config))
    }
}

impl WorkCalendar {
    /// Creates a new `WorkCalendar` with default work days (Monday to Friday) and no holidays.
    ///
    /// # Examples
    ///
    /// ```
    /// use workdays::WorkCalendar;
    /// use chrono::Weekday;
    ///
    /// let calendar = WorkCalendar::new();
    /// assert!(calendar.is_work_day(&Weekday::Mon));
    /// assert!(!calendar.is_work_day(&Weekday::Sat));
    /// ```
    pub fn new() -> Self {
        let work_days = [
            Weekday::Mon,
            Weekday::Tue,
            Weekday::Wed,
            Weekday::Thu,
            Weekday::Fri,
        ]
        .iter()
        .cloned()
        .collect();
        WorkCalendar {
            work_days,
            holidays: HashSet::new(),
        }
    }

    /// Computes the end date and calendar duration given a start date and number of work days.
    ///
    /// # Arguments
    ///
    /// * `start_date` - The starting date.
    /// * `days_worked` - Number of work days to add.
    ///
    /// # Returns
    ///
    /// A tuple containing the end date and the calendar duration, or an error if the input is invalid.
    ///
    /// # Examples
    ///
    /// ```
    /// use workdays::WorkCalendar;
    /// use chrono::NaiveDate;
    ///
    /// let calendar = WorkCalendar::new();
    /// let start_date = NaiveDate::from_ymd_opt(2023, 8, 21).unwrap();
    /// let (end_date, duration) = calendar.compute_end_date(start_date, 5).unwrap();
    ///
    /// assert_eq!(end_date, NaiveDate::from_ymd_opt(2023, 8, 25).unwrap());
    /// assert_eq!(duration.num_days(), 4);
    /// ```
    pub fn compute_end_date(
        &self,
        start_date: NaiveDate,
        days_worked: i64,
    ) -> Result<(NaiveDate, Duration), String> {
        if days_worked < 0 {
            return Err("days_worked must be non-negative".to_string());
        }

        if self.work_days.is_empty() {
            return Err("No work days defined".to_string());
        }

        let mut current_date = start_date;
        let mut remaining_days = days_worked;

        // If the start date is a work day, count it
        if self.is_work_day(&current_date.weekday()) && !self.is_holiday(&current_date) {
            remaining_days -= 1;
        }

        while remaining_days > 0 {
            current_date += Duration::days(1);

            if self.is_work_day(&current_date.weekday()) && !self.is_holiday(&current_date) {
                remaining_days -= 1;
            }
        }

        let calendar_duration = current_date.signed_duration_since(start_date);
        Ok((current_date, calendar_duration))
    }

    /// Adds a work day to the calendar.
    ///
    /// # Arguments
    ///
    /// * `day` - The `Weekday` to add as a work day.
    ///
    /// # Examples
    ///
    /// ```
    /// use workdays::WorkCalendar;
    /// use chrono::Weekday;
    ///
    /// let mut calendar = WorkCalendar::new();
    /// calendar.add_work_day(Weekday::Sat);
    /// assert!(calendar.is_work_day(&Weekday::Sat));
    /// ```
    pub fn add_work_day(&mut self, day: Weekday) {
        self.work_days.insert(day);
    }

    /// Removes a work day from the calendar.
    ///
    /// # Arguments
    ///
    /// * `day` - The `Weekday` to remove from work days.
    ///
    /// # Examples
    ///
    /// ```
    /// use workdays::WorkCalendar;
    /// use chrono::Weekday;
    ///
    /// let mut calendar = WorkCalendar::new();
    /// calendar.remove_work_day(&Weekday::Fri);
    /// assert!(!calendar.is_work_day(&Weekday::Fri));
    /// ```
    pub fn remove_work_day(&mut self, day: &Weekday) {
        self.work_days.remove(day);
    }

    /// Adds a holiday to the calendar.
    ///
    /// # Arguments
    ///
    /// * `date` - The `NaiveDate` to add as a holiday.
    ///
    /// # Examples
    ///
    /// ```
    /// use workdays::WorkCalendar;
    /// use chrono::NaiveDate;
    ///
    /// let mut calendar = WorkCalendar::new();
    /// let holiday = NaiveDate::from_ymd_opt(2023, 12, 25).unwrap();
    /// calendar.add_holiday(holiday);
    /// assert!(calendar.is_holiday(&holiday));
    /// ```
    pub fn add_holiday(&mut self, date: NaiveDate) {
        self.holidays.insert(date);
    }

    /// Removes a holiday from the calendar.
    ///
    /// # Arguments
    ///
    /// * `date` - The `NaiveDate` to remove from holidays.
    ///
    /// # Examples
    ///
    /// ```
    /// use workdays::WorkCalendar;
    /// use chrono::NaiveDate;
    ///
    /// let mut calendar = WorkCalendar::new();
    /// let holiday = NaiveDate::from_ymd_opt(2023, 12, 25).unwrap();
    /// calendar.add_holiday(holiday);
    /// calendar.remove_holiday(&holiday);
    /// assert!(!calendar.is_holiday(&holiday));
    /// ```
    pub fn remove_holiday(&mut self, date: &NaiveDate) {
        self.holidays.remove(date);
    }

    /// Set work days from a comma-separated string.
    ///
    /// This method overrides all existing work days with the ones specified in the input string.
    /// Day names are case-insensitive and can be full names (e.g., "Monday") or abbreviations (e.g., "Mon").
    ///
    /// # Arguments
    ///
    /// * `days` - A comma-separated string of day names (e.g., "mon,tue,Wednesday,thu,friday")
    ///
    /// # Returns
    ///
    /// * `Ok(())` if successful
    /// * `Err(String)` if the input string contains invalid day names
    ///
    /// # Examples
    ///
    /// ```
    /// use workdays::WorkCalendar;
    /// use chrono::Weekday;
    ///
    /// let mut calendar = WorkCalendar::new();
    /// calendar.set_work_days("Mon,Wed,Fri").unwrap();
    /// assert!(calendar.is_work_day(&Weekday::Mon));
    /// assert!(calendar.is_work_day(&Weekday::Wed));
    /// assert!(calendar.is_work_day(&Weekday::Fri));
    /// assert!(!calendar.is_work_day(&Weekday::Tue));
    /// ```
    pub fn set_work_days(&mut self, days: &str) -> Result<(), String> {
        let new_work_days: HashSet<Weekday> = days
            .split(',')
            .filter_map(|day| parse_weekday(day.trim()))
            .collect();

        if new_work_days.is_empty() {
            return Err("No valid work days provided".to_string());
        }

        self.work_days = new_work_days;
        Ok(())
    }

    /// Checks if a given day is a work day.
    ///
    /// # Arguments
    ///
    /// * `day` - The `Weekday` to check.
    ///
    /// # Returns
    ///
    /// `true` if the day is a work day, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use workdays::WorkCalendar;
    /// use chrono::Weekday;
    ///
    /// let calendar = WorkCalendar::new();
    /// assert!(calendar.is_work_day(&Weekday::Mon));
    /// assert!(!calendar.is_work_day(&Weekday::Sat));
    /// ```
    pub fn is_work_day(&self, day: &Weekday) -> bool {
        self.work_days.contains(day)
    }

    /// Checks if a given date is a holiday.
    ///
    /// # Arguments
    ///
    /// * `date` - The `NaiveDate` to check.
    ///
    /// # Returns
    ///
    /// `true` if the date is a holiday, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use workdays::WorkCalendar;
    /// use chrono::NaiveDate;
    ///
    /// let mut calendar = WorkCalendar::new();
    /// let holiday = NaiveDate::from_ymd_opt(2023, 12, 25).unwrap();
    /// calendar.add_holiday(holiday);
    /// assert!(calendar.is_holiday(&holiday));
    /// ```
    pub fn is_holiday(&self, date: &NaiveDate) -> bool {
        self.holidays.contains(date)
    }

    /// Calculates the number of work days between two dates (inclusive).
    ///
    /// # Arguments
    ///
    /// * `start_date` - The starting date.
    /// * `end_date` - The ending date.
    ///
    /// # Returns
    ///
    /// The number of work days between the two dates (inclusive).
    ///
    /// # Examples
    ///
    /// ```
    /// use workdays::WorkCalendar;
    /// use chrono::NaiveDate;
    ///
    /// let calendar = WorkCalendar::new();
    /// let start_date = NaiveDate::from_ymd_opt(2023, 8, 21).unwrap(); // Monday
    /// let end_date = NaiveDate::from_ymd_opt(2023, 8, 25).unwrap();   // Friday
    /// assert_eq!(calendar.work_days_between(start_date, end_date), 5);
    /// ```
    pub fn work_days_between(&self, start_date: NaiveDate, end_date: NaiveDate) -> i64 {
        let mut work_days = 0;
        let mut current_date = start_date;

        while current_date <= end_date {
            if self.work_days.contains(&current_date.weekday()) && !self.is_holiday(&current_date) {
                work_days += 1;
            }
            current_date += Duration::days(1);
        }

        work_days
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct WorkCalendarConfig {
    work_days: Option<Vec<String>>,
    holidays: Option<Vec<String>>,
}

impl From<WorkCalendarConfig> for WorkCalendar {
    fn from(config: WorkCalendarConfig) -> Self {
        let mut calendar = WorkCalendar::new();

        if let Some(days) = config.work_days {
            calendar.work_days = days
                .into_iter()
                .filter_map(|day| parse_weekday(&day))
                .collect();
        }

        if let Some(dates) = config.holidays {
            calendar.holidays = dates
                .into_iter()
                .filter_map(|date_str| NaiveDate::parse_from_str(&date_str, "%Y-%m-%d").ok())
                .collect();
        }

        calendar
    }
}

/// Parses a weekday string into a `Weekday` enum.
///
/// This function is case-insensitive and accepts both full names (e.g., "Monday")
/// and abbreviations (e.g., "Mon").
///
/// # Arguments
///
/// * `day` - A string slice that holds the name of the day.
///
/// # Returns
///
/// * `Some(Weekday)` if the day name is valid.
/// * `None` if the day name is invalid.
///
/// # Examples
///
/// ```
/// use workdays::parse_weekday;
/// use chrono::Weekday;
///
/// assert_eq!(parse_weekday("Monday"), Some(Weekday::Mon));
/// assert_eq!(parse_weekday("tue"), Some(Weekday::Tue));
/// assert_eq!(parse_weekday("Invalid"), None);
/// ```
pub fn parse_weekday(day: &str) -> Option<Weekday> {
    match day.to_lowercase().as_str() {
        "monday" | "mon" => Some(Weekday::Mon),
        "tuesday" | "tue" => Some(Weekday::Tue),
        "wednesday" | "wed" => Some(Weekday::Wed),
        "thursday" | "thu" => Some(Weekday::Thu),
        "friday" | "fri" => Some(Weekday::Fri),
        "saturday" | "sat" => Some(Weekday::Sat),
        "sunday" | "sun" => Some(Weekday::Sun),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_end_date_standard_week() {
        let calendar = WorkCalendar::new();
        let start_date = NaiveDate::from_ymd_opt(2023, 8, 21).unwrap(); // Monday
        let (end_date, duration) = calendar.compute_end_date(start_date, 5).unwrap();

        assert_eq!(end_date, NaiveDate::from_ymd_opt(2023, 8, 25).unwrap()); // Friday
        assert_eq!(duration.num_days(), 4);
    }

    #[test]
    fn test_compute_end_date_with_weekend() {
        let calendar = WorkCalendar::new();
        let start_date = NaiveDate::from_ymd_opt(2023, 8, 18).unwrap(); // Friday
        let (end_date, duration) = calendar.compute_end_date(start_date, 5).unwrap();

        assert_eq!(end_date, NaiveDate::from_ymd_opt(2023, 8, 24).unwrap()); // Next Thursday
        assert_eq!(duration.num_days(), 6);
    }

    #[test]
    fn test_compute_end_date_with_holiday() {
        let mut calendar = WorkCalendar::new();
        calendar.add_holiday(NaiveDate::from_ymd_opt(2023, 8, 23).unwrap()); // Wednesday
        let start_date = NaiveDate::from_ymd_opt(2023, 8, 21).unwrap(); // Monday
        let (end_date, duration) = calendar.compute_end_date(start_date, 5).unwrap();

        assert_eq!(end_date, NaiveDate::from_ymd_opt(2023, 8, 28).unwrap()); // Next Monday
        assert_eq!(duration.num_days(), 7);
    }

    #[test]
    fn test_compute_end_date_custom_work_days() {
        let mut calendar = WorkCalendar::new();
        calendar.set_work_days("Monday,Wednesday,Friday").unwrap();
        let start_date = NaiveDate::from_ymd_opt(2023, 8, 21).unwrap(); // Monday
        let (end_date, duration) = calendar.compute_end_date(start_date, 5).unwrap();

        assert_eq!(end_date, NaiveDate::from_ymd_opt(2023, 8, 30).unwrap()); // Wednesday
        assert_eq!(duration.num_days(), 9);
    }

    #[test]
    fn test_compute_end_date_zero_days() {
        let calendar = WorkCalendar::new();
        let start_date = NaiveDate::from_ymd_opt(2023, 8, 21).unwrap(); // Monday
        let (end_date, duration) = calendar.compute_end_date(start_date, 0).unwrap();

        assert_eq!(end_date, start_date);
        assert_eq!(duration.num_days(), 0);
    }

    #[test]
    fn test_compute_end_date_negative_days() {
        let calendar = WorkCalendar::new();
        let start_date = NaiveDate::from_ymd_opt(2023, 8, 21).unwrap(); // Monday
        assert!(calendar.compute_end_date(start_date, -1).is_err());
    }

    #[test]
    fn test_set_work_days() {
        let mut calendar = WorkCalendar::new();
        assert!(calendar.set_work_days("Mon,Wed,Fri").is_ok());
        assert!(calendar.is_work_day(&Weekday::Mon));
        assert!(calendar.is_work_day(&Weekday::Wed));
        assert!(calendar.is_work_day(&Weekday::Fri));
        assert!(!calendar.is_work_day(&Weekday::Tue));
        assert!(!calendar.is_work_day(&Weekday::Thu));
        assert!(!calendar.is_work_day(&Weekday::Sat));
        assert!(!calendar.is_work_day(&Weekday::Sun));
    }

    #[test]
    fn test_add_and_remove_holiday() {
        let mut calendar = WorkCalendar::new();
        let holiday = NaiveDate::from_ymd_opt(2023, 12, 25).unwrap();
        calendar.add_holiday(holiday);
        assert!(calendar.is_holiday(&holiday));
        calendar.remove_holiday(&holiday);
        assert!(!calendar.is_holiday(&holiday));
    }

    #[test]
    fn test_work_days_between() {
        let calendar = WorkCalendar::new();
        let start_date = NaiveDate::from_ymd_opt(2023, 8, 21).unwrap(); // Monday
        let end_date = NaiveDate::from_ymd_opt(2023, 8, 25).unwrap(); // Friday
        assert_eq!(calendar.work_days_between(start_date, end_date), 5);

        let end_date = NaiveDate::from_ymd_opt(2023, 8, 27).unwrap(); // Sunday
        assert_eq!(calendar.work_days_between(start_date, end_date), 5);

        let mut calendar = WorkCalendar::new();
        calendar.add_holiday(NaiveDate::from_ymd_opt(2023, 8, 23).unwrap()); // Wednesday
        assert_eq!(calendar.work_days_between(start_date, end_date), 4);
    }

    #[test]
    fn test_from_str_yaml() {
        let config = r#"
        work_days:
          - Monday
          - Wednesday
          - Friday
        holidays:
          - 2023-12-25
        "#;
        let calendar = WorkCalendar::from_str(config).unwrap();
        assert!(calendar.is_work_day(&Weekday::Mon));
        assert!(calendar.is_work_day(&Weekday::Wed));
        assert!(calendar.is_work_day(&Weekday::Fri));
        assert!(!calendar.is_work_day(&Weekday::Tue));
        assert!(calendar.is_holiday(&NaiveDate::from_ymd_opt(2023, 12, 25).unwrap()));
    }

    #[test]
    fn test_from_str_json() {
        let config = r#"
        {
            "work_days": ["Monday", "Wednesday", "Friday"],
            "holidays": ["2023-12-25"]
        }
        "#;
        let calendar = WorkCalendar::from_str(config).unwrap();
        assert!(calendar.is_work_day(&Weekday::Mon));
        assert!(calendar.is_work_day(&Weekday::Wed));
        assert!(calendar.is_work_day(&Weekday::Fri));
        assert!(!calendar.is_work_day(&Weekday::Tue));
        assert!(calendar.is_holiday(&NaiveDate::from_ymd_opt(2023, 12, 25).unwrap()));
    }
}
