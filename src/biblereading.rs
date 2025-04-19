/// In this unit, all the logic for the bible reading references is going to be implemented.

use core::fmt;

use chrono::{Datelike, Local, NaiveDate};

#[derive(Debug, Clone)]
pub struct BibleReading {
    pub date: NaiveDate,
    pub old_testament_reading: String,
    pub new_testament_reading: String,
}   

#[derive(Debug, Clone)]
enum ErrorCause {
    InputFileNotFound,
    DateDoesNotExist,
    InvalidFormat,
}

#[derive(Debug, Clone)]
pub struct BibleReadingNotFoundError {
    error_cause: ErrorCause,
    error_string: String,
}

impl BibleReadingNotFoundError {
    fn new(error_cause: ErrorCause) -> BibleReadingNotFoundError {
        BibleReadingNotFoundError {
            error_cause,
            error_string: String::from(""),
        }
    }
}

impl fmt::Display for BibleReadingNotFoundError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.error_cause {
            ErrorCause::DateDoesNotExist => write!(f, "There exists no entry with bible reading for today's date."),
            ErrorCause::InputFileNotFound => write!(f, "The input file has not been found."),
            ErrorCause::InvalidFormat => write!(f, "The format of the csv file seems to be invalid: {}", self.error_string)
        }
    }
}

pub fn get_todays_biblereading() -> Result<BibleReading, BibleReadingNotFoundError> {
    let today: NaiveDate = Local::now().date_naive();
    get_biblereading_for_date(today)
}

fn get_biblereading_for_date(search_date: NaiveDate) -> Result<BibleReading, BibleReadingNotFoundError> {
    // We use the default year 2000 as the year does not matter in an annual schedule.
    const DEFAULT_YEAR: i32 = 2000;
    
    // Change the year in the search_date
    let search_date: NaiveDate = search_date.with_year(DEFAULT_YEAR).unwrap();

    let csv_reader_result = csv::Reader::from_path("input/schedule.csv");
    if csv_reader_result.is_err() {
        return Err(BibleReadingNotFoundError::new(ErrorCause::InputFileNotFound));
    }
    let csv_reader = csv_reader_result.unwrap();
    dbg!("Approach CSV Reader");
    for record in csv_reader.into_records() {
        match record {
            Ok(string_record) => {
                if string_record.len() != 3 {
                    return Err(BibleReadingNotFoundError {
                        error_cause: ErrorCause::InvalidFormat,
                        error_string: "The length of the row is not always 3".to_string()
                    });
                }

                match NaiveDate::parse_from_str(&format!("{}-{}", string_record.get(0).unwrap(), DEFAULT_YEAR), "%m-%d-%Y") {
                    // The date can be parsed from string and we have a NaiveDate
                    Ok(date) => {
                        let normalized_date = date.with_year(DEFAULT_YEAR).unwrap();
                        if normalized_date == search_date {
                            return Ok(
                                BibleReading {
                                    date: normalized_date,
                                    old_testament_reading: string_record.get(2).unwrap().to_string(),
                                    new_testament_reading: string_record.get(1).unwrap().to_string(),
                                }
                            )
                        }
                    },
                    // The date can not be parsed from string (most likely because of an invalid format)
                    Err(_) => { 
                        dbg!("Error!");
                        dbg!(string_record.get(0).unwrap());

                        return Err(BibleReadingNotFoundError {
                            error_cause: ErrorCause::InvalidFormat,
                            error_string: format!("Can not parse date {}", string_record.get(0).unwrap())
                        })
                    }
                }
            },
            Err(_) => { }
        }
    }

    // If nothing has been found, we return an DateDoesNotExist Error
    Err(BibleReadingNotFoundError {
        error_cause: ErrorCause::DateDoesNotExist,
        error_string: String::from("")
    })
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn date_can_be_found() {
        let search_result = get_biblereading_for_date(NaiveDate::from_ymd_opt(2024, 9, 1).unwrap());
        assert!(search_result.is_ok());
        
        let biblereading = search_result.unwrap();
        assert_eq!(biblereading.old_testament_reading, "Psalm 135,136");
        assert_eq!(biblereading.new_testament_reading, "1Kor12");
    }
}
    
