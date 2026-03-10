use approx::relative_eq;
use chrono::{DateTime, Datelike, Duration, FixedOffset, NaiveDate, Utc};
use solar_positioning::SunriseResult;
use solar_positioning::spa;
use solar_positioning::time::DeltaT;
use std::time::Instant;

fn main() {
    // Ефремов 53.149159, 38.121840
    const TIME_OFFSET_SECONDS: i32 = 10800; //3 * 3600 or Moscow time
    const LAT: f64 = 53.149159; //Efremov
    const LON: f64 = 38.121840; //Efremov
    let start = Instant::now(); //Timestamp for performance check

    let today = Utc::now(); //current date and time
    // For tests uncomment below "let today"
    //let today: DateTime<Utc> = "2026-06-22T00:00:00+00:00".parse().expect("Incorrect date format");
    let timezone = FixedOffset::east_opt(TIME_OFFSET_SECONDS); //Set timezone
    let delta_t = DeltaT::estimate_from_date_like(today).unwrap_or(69.0); //Delta T from date

    // Вычисление sunrise / sunset (горизонт 0° + рефракция)
    let res_today = spa::sunrise_sunset_for_horizon(
        today,
        LAT,
        LON,
        delta_t,
        solar_positioning::Horizon::SunriseSunset, // стандартный горизонт с рефракцией
    )
    .expect("Error. Cannot calculate today spa");

    println!("Today {}", today.format("%Y-%m-%d"));

    print_days_to_ny(today);

    //Main calculations
    let mut daylength: f32 = 0.0; //set initial day length

    match res_today {
        SunriseResult::RegularDay {
            sunrise,
            transit,
            sunset,
        } => {
            print_today(&timezone, sunrise, transit, sunset); //Long code moved to separate function. For further improvements
            daylength = time_diff(sunrise, sunset);
            println!("{:<12}{}", "Daylength:", seconds_to_hms(daylength));
            let to_sunset = time_diff(today, sunset);
            if to_sunset <= 0.0 {
                println!("Sun is below the horizon now")
            } else {
                println!("{:<12}{}", "To sunset:", seconds_to_hms(to_sunset));
            }
        }
        _ => println!("No sunrise or sunset today"), //If no sunrise or sunset happened. However, for Efremov it's impossible
    }

    let yesterday = today - Duration::days(1);
    let tomorrow = today + Duration::days(1);

    let res_yesterday = spa::sunrise_sunset_for_horizon(
        yesterday,
        LAT,
        LON,
        delta_t,
        solar_positioning::Horizon::SunriseSunset,
    )
    .expect("Error. Cannot calculate yesterday spa");

    let res_tomorrow = spa::sunrise_sunset_for_horizon(
        tomorrow,
        LAT,
        LON,
        delta_t,
        solar_positioning::Horizon::SunriseSunset,
    )
    .expect("Error. Cannot calculate tomorrow spa");

    let daylength_yesterday =
        (*res_yesterday.sunset().unwrap() - res_yesterday.sunrise().unwrap()).as_seconds_f32();
    let daylength_tomorrow =
        (*res_tomorrow.sunset().unwrap() - res_tomorrow.sunrise().unwrap()).as_seconds_f32();

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

    let day_diff = daylength - daylength_yesterday; //Difference between yesterday and today daylength
    let verb = if day_diff < 0.0 { "shorter" } else { "longer" };
    println!(
        "{:<22} {}",
        format!("Today is {} by:", verb),
        seconds_to_hms(day_diff.abs()) //Passing absolute value to function
    );

    find_next_date(LAT, LON, today, delta_t, daylength); //long slice of code introduced as function
    let finish = start.elapsed(); //For performance check
    println!("Calculations took: {} seconds", finish.as_secs_f64())
}

fn print_days_to_ny(today: DateTime<Utc>) {
    let to_ny = days_to_new_year(today); //Days to new year
    let days_in_year = if is_leap_year(today.year()) { 366 } else { 365 }; //amount days in year
    if to_ny == 0 {
        //To make people happy
        println!("Today is New Year! Happy holidays!")
    } else {
        println!(
            "Days to New Year: {}. Year completed at {:.02}%",
            to_ny,
            100.0 - (to_ny as f32 / days_in_year as f32 * 100.0)
        )
    }
}

fn print_today(
    timezone: &Option<FixedOffset>,
    sunrise: DateTime<Utc>,
    transit: DateTime<Utc>,
    sunset: DateTime<Utc>,
) {
    println!(
        "{:<12}{}\r\n{:<12}{}\r\n{:<12}{}",
        "Sunrise:",
        sunrise.with_timezone(&timezone.unwrap()).format("%H:%M:%S"),
        "Solar noon:",
        transit.with_timezone(&timezone.unwrap()).format("%H:%M:%S"),
        "Sunset:",
        sunset.with_timezone(&timezone.unwrap()).format("%H:%M:%S")
    );
}

fn find_next_date(lat: f64, lon: f64, today: DateTime<Utc>, delta_t: f64, daylength: f32) {
    let mut dl_list: Vec<(f32, DateTime<Utc>)> = Vec::new();
    for d in 1..365 {
        let next_date = today + Duration::days(d);
        let future = spa::sunrise_sunset_for_horizon(
            next_date,
            lat,
            lon,
            delta_t,
            solar_positioning::Horizon::SunriseSunset,
        )
        .expect("Error. Cannot calculate future spa");
        match future {
            SunriseResult::RegularDay {
                sunrise,
                transit: _,
                sunset,
            } => {
                let next_length = time_diff(sunrise, sunset);
                if relative_eq!(next_length, daylength, epsilon = 100.0) {
                    dl_list.push((next_length, next_date));
                }
            }
            _ => println!("No sunrise or sunset today"),
        }
    }
    if dl_list.is_empty() {
        //if somehow mirror value not found
        println!("Mirror date not found. Try bigger epsilon");
        std::process::exit(1); //terminating program
    }
    let mirror_date = get_mirror(daylength, &dl_list);
    println!(
        "Next date with almost same length is {}. Day length will be {}",
        mirror_date.1.format("%Y-%m-%d"),
        seconds_to_hms(mirror_date.0)
    );
}

fn get_mirror(
    //seek mirror date based on day length
    daylength: f32,
    dl_list: &[(f32, DateTime<Utc>)],
) -> (f32, DateTime<Utc>) {
    dl_list
        .iter()
        .min_by(|a, b| {
            let da = (a.0 - daylength).abs();
            let db = (b.0 - daylength).abs();
            da.partial_cmp(&db).unwrap()
        })
        .copied()
        .unwrap()
}

fn days_to_new_year(dt: DateTime<Utc>) -> u16 {
    //calculates how many days left to New Year/ Written by claude
    let current_year = dt.year();
    let next_year = current_year + 1;
    let new_year = NaiveDate::from_ymd_opt(next_year, 1, 1).expect("Invalid date");
    (new_year - dt.date_naive()).num_days() as u16
}
fn time_diff(time1: DateTime<Utc>, time2: DateTime<Utc>) -> f32 {
    //return time difference between two dates
    (time2 - time1).as_seconds_f32()
}

fn seconds_to_hms(total_seconds: f32) -> String {
    //written by claude. convert value into hours, minutes and seconds.
    let total_seconds = total_seconds as u32;
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;

    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}

fn is_leap_year(year: i32) -> bool {
    NaiveDate::from_ymd_opt(year, 2, 29).is_some()
} //written by claude
