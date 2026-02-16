use chrono::{DateTime, Datelike, Timelike, Utc};
use std::f64::consts::PI;

const LAT: f64 = 53.150058;
const LON: f64 = 38.121094;
const ELEVATION: i16 = 174;

struct Dateonly {
    year: u16,
    month: u8,
    day: u8,
}

fn main() {
    let today = chrono::Utc::now();
    let date = parse_date(today);
    let jd = julian_date(date.year, date.month, date.day, today.hour());
    let decl = sun_declination(jd);

    println!("Today {}-{}-{}", date.year, date.month, date.day);
    println!("Julian date: {}", jd);
    println!("Sun declination: {}", decl);
}

fn julian_date(year: u16, month: u8, day: u8, ut_hours: u32) -> f64 {
    let y = year as f64;
    let m = month as f64;
    let d = day as f64;
    let ut_hours = ut_hours as f64;

    let a = ((14.0 - m) / 12.0).floor() as i32;
    let yy = (y as i32 + 4800 - a) as f64;
    let mm = (m as i32 + 12 * a - 3) as f64;

    let jdn = d.floor()
        + (153.0 * mm + 2.0).floor() / 5.0
        + 365.0 * yy
        + (yy / 4.0).floor()
        - (yy / 100.0).floor()
        + (yy / 400.0).floor()
        - 32045.0;

    jdn + (ut_hours - 12.0) / 24.0
}

fn sun_declination(jd: f64) -> f64 {
    let n = jd - 2451545.0;

    // Средняя аномалия Солнца (M) — лучше, чем просто L
    let m_deg = 357.5291 + 0.98560028 * n;
    let m = m_deg.to_radians();

    // Уравнение центра (более точное приближение)
    let c = 1.9148 * m.sin()
        + 0.0200 * (2.0 * m).sin()
        + 0.0003 * (3.0 * m).sin();

    // Средняя долгота Солнца
    let lambda_deg = 280.4665 + 0.98564736 * n + c;

    let epsilon = 23.4393 - 0.0000004 * n;  // наклон эклиптики, слегка корректируем

    let sin_delta = lambda_deg.to_radians().sin() * epsilon.to_radians().sin();
    let delta_rad = sin_delta.asin();

    delta_rad.to_degrees()  // возвращаем градусы — удобнее
}

fn parse_date(date: DateTime<Utc>) -> Dateonly {
    Dateonly {year: date.year() as u16, month: date.month() as u8, day: date.day() as u8}
}
