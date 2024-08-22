# Workdays

`workdays` is a Rust library for computing work days and handling work
calendars, inspired by the WORKDAY function in Excel. It provides functionality
to define custom work weeks, add holidays, and compute dates based on a given
number of work days.

## Features

- Compute end dates based on work days
- Calculate the number of work days between two dates
- Parse and handle work calendar configurations (YAML or JSON)
- Support for custom work days and holidays
- Flexible weekday parsing

## Installation

Add `workdays` as a dependency in your `Cargo.toml`:

```bash
cargo add workdays
```

## Usage

Here's a quick example:

```rust
use workdays::WorkCalendar;
use chrono::NaiveDate;
use std::str::FromStr;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut calendar = WorkCalendar::new();
    calendar.add_holiday(NaiveDate::from_ymd_opt(2023, 12, 25).unwrap());
    calendar.set_work_days("Mon,Tue,Wed,Thu,Fri")?;

    let start_date = NaiveDate::from_ymd_opt(2023, 8, 21).unwrap();
    let days_worked = 20;

    let (end_date, calendar_duration) = calendar.compute_end_date(start_date, days_worked)?;
    println!("End date: {}", end_date);
    println!("Calendar duration: {} days", calendar_duration.num_days());

    // Using FromStr to create a WorkCalendar from YAML or JSON
    let config = r#"
    work_days:
      - Monday
      - Tuesday
      - Wednesday
    holidays:
      - 2023-12-25
    "#;
    let custom_calendar = WorkCalendar::from_str(config)?;
    println!("Is Monday a work day? {}", custom_calendar.is_work_day(&chrono::Weekday::Mon));

    Ok(())
}
```

## License

This project is licensed under the MIT License - see the [LICENSE](./LICENSE) file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Acknowledgments

- Inspired by the WORKDAY function in Excel
