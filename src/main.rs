#![allow(non_snake_case)]
extern crate exif;
use exif::{DateTime, Value, Tag};
use std::env;
use std::io::prelude::*;
use std::fs;
use std::fs::File;
use std::time::{UNIX_EPOCH};
const LOG_FILE_NAME:&'static str = "Archiver Log.txt";
struct DateInfo {
    year: u64,
    month: u64,
    day: u64,
    exif: bool,
}
fn main() {
    let months = vec!["99 Unknown", "01 January", "02 February", "03 March", "04 April", "05 May", "06 June", "07 July", "08 August", "09 September", "10 October", "11 November", "12 December"];
    let mut path = env::current_dir().unwrap(); //Set current directory as working directory.
    &path.push(LOG_FILE_NAME);
    let mut log_file = File::create(&path).unwrap();
    &path.pop();
    let mut processed_file_count: u64 = 0;
    for entry in fs::read_dir(&path).unwrap() { //Iterate over the directory.
        if let Ok(entry) = entry {
            if let Ok(meta) = entry.metadata() { //Read entry metadata (file or folder).
                if meta.is_dir() {
                    log_file.write_all(format!("Skipping Directory: {}\n", entry.path().display()).as_bytes()).unwrap();
                    continue;
                }
                if meta.is_file() {
                    log_file.write_all(format!("Processing File: {}\n", &entry.path().display()).as_bytes()).unwrap();
                    let mut date_info = DateInfo {
                        year: 0,
                        month: 0,
                        day: 0,
                        exif: false,
                    };
                    let mut upper_extension = String::new();
                    let mut lower_extension = String::new();
                    let mut original_file_name = String::new();
                    if let Some(file_extension) = entry.path().extension() {
                        let temp = String::from(file_extension.to_str().unwrap());
                        &upper_extension.push_str(&temp.to_uppercase());
                        &lower_extension.push_str(&temp.to_lowercase());
                    }
                    if let Some(file_name) = entry.path().file_stem() {
                        let temp = String::from(file_name.to_str().unwrap());
                        &original_file_name.push_str(&temp);
                    }
                    match &upper_extension[..] {
                        "JPG" => date_info = extract_exif_info(&entry),
                        "JPEG" => date_info = extract_exif_info(&entry),
                        "TFF" => date_info = extract_exif_info(&entry),
                        "TIFF" => date_info = extract_exif_info(&entry),
                        "PNG" => {},
                        "MOV" => {},
                        "MP4" => {},
                        _ => continue,
                    }
                    if !date_info.exif {
                        if let Ok(st) = meta.modified() {
                            let seconds = st.duration_since(UNIX_EPOCH).unwrap().as_secs();
                            date_info = break_time(seconds);
                        }
                        log_file.write_all(format!("Metada Extraction: {}/{}/{}\n", date_info.day, date_info.month, date_info.year).as_bytes()).unwrap();
                    } else {
                        log_file.write_all(format!("EXIF Extraction: {}/{}/{}\n", date_info.day, date_info.month, date_info.year).as_bytes()).unwrap();
                    }
                    &path.push(date_info.year.to_string());
                    &path.push(&months[date_info.month as usize]);
                    &path.push(date_info.day.to_string());
                    std::fs::create_dir_all(&path).unwrap();
                    &path.push(format!("{}_{}.{}", &original_file_name, processed_file_count, &lower_extension));
                    fs::rename(&entry.path(), &path).unwrap();
                    log_file.write_all(format!("File Moved Here: {}\n", &path.display()).as_bytes()).unwrap();
                    processed_file_count = processed_file_count + 1;
                    &path.pop();
                    &path.pop();
                    &path.pop();
                    &path.pop();
                } else {
                    log_file.write_all(format!("Skipping Item: {}\n", &entry.path().display()).as_bytes()).unwrap();
                }
            }
        }
    }
    log_file.write_all(format!("Processed Files: {}\n", processed_file_count).as_bytes()).unwrap();
}

fn break_time (time: u64) -> DateInfo {
    const EPOCH_YR: u64 = 1970;
    const SECS_DAY: u64 = 24 * 60 * 60;
    let mut year_table = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let mut result_year = EPOCH_YR;
    let mut result_month: u64 = 0;
    let mut result_day = time / SECS_DAY;

    //Useful but out of scope.
    //let result_weekday = (result_day + 4) % 7; //Day 0 was a thursday.
    //let day_clock = time % SECS_DAY;
    //let result_hour = day_clock / 3600;
    //let result_minutes = (day_clock % 3600) / 60;
    //let result_seconds = day_clock % 60;

    while result_day >= year_size(result_year) {
        result_day = result_day - year_size(result_year);
        result_year = result_year + 1;
    }
    if is_leap_year(result_year) {
        year_table[1] = 29;
    }
    while result_day >= year_table[result_month as usize] {
        result_day = result_day - year_table[result_month as usize];
        result_month = result_month + 1;
    }
    let date_info = DateInfo {
        year: result_year,
        month: result_month + 1,
        day: result_day + 1,
        exif: false,
    };
    date_info
}

fn is_leap_year (year: u64) -> bool {
    if year % 4 == 0 {
        if year % 100 == 0 {
            if year % 400 == 0 {
                return true;
            }
            return false;
        }
        return true;
    }
    false
}

fn year_size (year: u64) -> u64 {
    if is_leap_year(year) {
        366
    } else {
        365
    }
}

fn extract_exif_info (entry: &std::fs::DirEntry) -> DateInfo {
    let file = std::fs::File::open(&entry.path()).unwrap();
    let reader = exif::Reader::new(&mut std::io::BufReader::new(&file));
    match reader {
        Ok(exif) => {
            let mut year = 0;
            let mut month = 0;
            let mut day = 0;
            for f in exif.fields() {
                if f.tag.number() == 36867 {
                    if let Some(field) = exif.get_field(Tag::DateTime, false) {
                        match field.value {
                            Value::Ascii(ref vec) if !vec.is_empty() => {
                                if let Ok(datetime) = DateTime::from_ascii(vec[0]) {
                                    year = datetime.year;
                                    month = datetime.month;
                                    day = datetime.day;
                                }
                            },
                            _ => {},
                        }
                    }
                }
            }
            let mut date_info = DateInfo {
                year: year as u64,
                month: month as u64,
                day: day as u64,
                exif: true,
            };
            if date_info.year == 0 || date_info.month == 0 || date_info.day == 0 {
                date_info.exif = false;
            }
            return date_info;
        }
        Err(_) => {
            let date_info = DateInfo {
                year: 0,
                month: 0,
                day: 0,
                exif: false,
            };
            return date_info;
        }
    }
}
