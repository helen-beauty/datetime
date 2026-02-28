use approx::relative_eq;
use chrono::{DateTime, Datelike, Duration, FixedOffset, NaiveDate, Utc};
use solar_positioning::SunriseResult;
use solar_positioning::spa;
use solar_positioning::time::DeltaT;
use std::time::Instant;

fn main() {
    // Ефремов 53.149159, 38.121840
    let start = Instant::now();
    let lat = 53.149159;
    let lon = 38.121840;
    let today = Utc::now();
    //let today: DateTime<Utc> = "2026-01-10T00:00:00+00:00"
    //  .parse()
    //.expect("Неверный формат даты");
    let timezone = FixedOffset::east_opt(3 * 3600);
    let delta_t = DeltaT::estimate_from_date_like(today).unwrap_or(69.0);

    // Вычисление sunrise / sunset (горизонт 0° + рефракция)
    let result = spa::sunrise_sunset_for_horizon(
        today,
        lat,
        lon,
        delta_t,
        solar_positioning::Horizon::SunriseSunset, // стандартный горизонт с рефракцией
    )
    .expect("Error 1");

    println!("Today {}", today.format("%Y-%m-%d"));
    let to_ny = days_to_new_year(today);
    let days_in_year = if is_leap_year(today.year()) { 366 } else { 365 };
    if to_ny == 0 {
        println!("Today is New Year! Happy holidays!")
    } else {
        println!(
            "Days to New Year: {}. Year completed at {:.02}%",
            to_ny,
            100.0 - (to_ny as f32 / days_in_year as f32 * 100.0)
        )
    }
    let mut daylength = 0.0;

    match result {
        SunriseResult::RegularDay {
            sunrise,
            transit,
            sunset,
        } => {
            println!(
                "{:<12}{}\r\n{:<12}{}\r\n{:<12}{}",
                "Sunrise:",
                sunrise
                    .with_timezone(&timezone.unwrap())
                    .format("%H:%M:%S")
                    .to_string(),
                "Solar noon:",
                transit
                    .with_timezone(&timezone.unwrap())
                    .format("%H:%M:%S")
                    .to_string(),
                "Sunset:",
                sunset
                    .with_timezone(&timezone.unwrap())
                    .format("%H:%M:%S")
                    .to_string()
            );
            daylength = time_diff(sunrise, sunset);
            println!("{:<12}{}", "Daylength:", seconds_to_hms(daylength));
            let to_sunset = time_diff(today, sunset);
            if to_sunset <= 0.0 {
                println!("Sun is below the horizon now")
            } else {
                println!("{:<12}{}", "To sunset:", seconds_to_hms(to_sunset));
            }
        }
        _ => println!("No sunrise or sunset today"),
    }

    let yesterday = today - Duration::days(1);
    let tomorrow = today + Duration::days(1);

    let res_yesterday = spa::sunrise_sunset_for_horizon(
        yesterday,
        lat,
        lon,
        delta_t,
        solar_positioning::Horizon::SunriseSunset,
    )
    .expect("Error 3");

    let res_tomorrow = spa::sunrise_sunset_for_horizon(
        tomorrow,
        lat,
        lon,
        delta_t,
        solar_positioning::Horizon::SunriseSunset,
    )
    .expect("Error 4");

    let daylength_yesterday =
        (*res_yesterday.sunset().unwrap() - res_yesterday.sunrise().unwrap()).as_seconds_f32();
    let daylength_tomorrow =
        (*res_tomorrow.sunset().unwrap() - res_tomorrow.sunrise().unwrap()).as_seconds_f32();
    let day_diff = daylength_tomorrow - daylength;

    println!(
        "{:<22} {}",
        "Day length yesterday:",
        seconds_to_hms(daylength_yesterday)
    );
    println!(
        "{:<22} {}",
        "Day length tomorrow:",
        seconds_to_hms(daylength_tomorrow)
    );

    if day_diff < 0.0 {
        println!(
            "{:<22} {}",
            "Today is shorter by:",
            seconds_to_hms(day_diff.abs())
        );
    } else {
        println!("{:<22} {}", "Today is longer by:", seconds_to_hms(day_diff));
    }

    find_next_date(lat, lon, today, delta_t, &mut daylength);
    let finish = start.elapsed();
    println!("Calculations took: {:?} seconds", finish.as_secs_f64())
}

fn find_next_date(lat: f64, lon: f64, today: DateTime<Utc>, delta_t: f64, daylength: &mut f32) {
    for d in 1..365 {
        let next_date = today + Duration::days(d);
        let future = spa::sunrise_sunset_for_horizon(
            next_date,
            lat,
            lon,
            delta_t,
            solar_positioning::Horizon::SunriseSunset,
        )
        .expect("Error 2");
        match future {
            SunriseResult::RegularDay {
                sunrise,
                transit: _,
                sunset,
            } => {
                let next_length = time_diff(sunrise, sunset);
                if relative_eq!(next_length, daylength, epsilon = 100.0) {
                    println!(
                        "Next date with almost same length is {}. Day length will be {}",
                        next_date.format("%Y-%m-%d"),
                        seconds_to_hms(next_length)
                    );
                }
            }
            _ => println!("No sunrise or sunset today"),
        }
    }
}

fn days_to_new_year(dt: DateTime<Utc>) -> u16 {
    let current_year = dt.year();
    let next_year = current_year + 1;
    let new_year = NaiveDate::from_ymd_opt(next_year, 1, 1).expect("Invalid date");
    let days_remaining = (new_year - dt.date_naive()).num_days() as u16;
    days_remaining
}
fn time_diff(sunrise: DateTime<Utc>, sunset: DateTime<Utc>) -> f32 {
    (sunset - sunrise).as_seconds_f32()
}

fn seconds_to_hms(total_seconds: f32) -> String {
    //written by claude
    let total_seconds = total_seconds as u32;
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;

    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}

fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}
