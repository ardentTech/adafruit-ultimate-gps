use chrono::{NaiveDate, NaiveTime};
use nmea::sentences::rmc::{RmcNavigationStatus, RmcStatusOfFix};
use nmea::sentences::{FaaMode, FixType, GgaData, GllData, RmcData};

/// GPS data composed of GGA, RMC and GLL payloads.
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug)]
pub struct Reading {
    pub altitude: Option<f32>,
    pub faa_mode: Option<FaaMode>,
    #[cfg_attr(feature = "defmt", defmt(Debug2Format))]
    pub fix_date: Option<NaiveDate>,
    pub fix_satellites: Option<u32>,
    #[cfg_attr(feature = "defmt", defmt(Debug2Format))]
    pub fix_time: Option<NaiveTime>,
    pub fix_type: Option<FixType>, // TODO remove this?
    pub geoid_separation: Option<f32>,
    pub hdop: Option<f32>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub magnetic_variation: Option<f32>,
    pub nav_status: Option<RmcNavigationStatus>,
    pub speed_over_ground: Option<f32>,
    pub status_of_fix: RmcStatusOfFix,
    pub true_course: Option<f32>,
    pub valid: bool,
}

impl Reading {

    /// Constructs a new instance prioritizing GGA data over RMC data over GLL data.
    pub fn new(gga: GgaData, gll: GllData, rmc: RmcData) -> Self {
        // GGA
        let mut reading = Self {
            altitude: gga.altitude,
            faa_mode: None,
            fix_date: None,
            fix_satellites: gga.fix_satellites,
            fix_time: gga.fix_time,
            fix_type: gga.fix_type,
            geoid_separation: gga.geoid_separation,
            hdop: gga.hdop,
            latitude: gga.latitude,
            longitude: gga.longitude,
            magnetic_variation: None,
            nav_status: None,
            speed_over_ground: None,
            status_of_fix: rmc.status_of_fix,
            true_course: None,
            valid: gll.valid,
        };

        // RMC
        if reading.fix_time.is_none() {
            reading.fix_time = rmc.fix_time;
        }
        if reading.latitude.is_none() {
            reading.latitude = rmc.lat;
        }
        if reading.longitude.is_none() {
            reading.longitude = rmc.lon;
        }
        if reading.faa_mode.is_none() {
            reading.faa_mode = rmc.faa_mode;
        }
        reading.fix_date = rmc.fix_date;
        reading.speed_over_ground = rmc.speed_over_ground;
        reading.true_course = rmc.true_course;
        reading.magnetic_variation = rmc.magnetic_variation;
        reading.nav_status = rmc.nav_status;

        // GLL
        if reading.fix_time.is_none() {
            reading.fix_time = gll.fix_time;
        }
        if reading.latitude.is_none() {
            reading.latitude = gll.latitude;
        }
        if reading.longitude.is_none() {
            reading.longitude = gll.longitude;
        }
        reading.faa_mode = gll.faa_mode;
        reading
    }
}