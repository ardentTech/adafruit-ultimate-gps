use chrono::{NaiveDate, NaiveTime};
use core::str::Utf8Error;
#[cfg(feature = "defmt")]
use defmt::Format;
use heapless::String;
use nmea::ParseResult;
use nmea::sentences::rmc::{RmcNavigationStatus, RmcStatusOfFix};
use nmea::sentences::{FaaMode, FixType, GgaData, GllData, RmcData};
use pmtk::error::PmtkError;
use pmtk::response::PmtkResponse;

pub const SENTENCE_MAX_LEN: usize = 255;

pub type RawSentence = String<SENTENCE_MAX_LEN>;

#[derive(Debug, PartialEq)]
pub enum GpsResponse {
    Nmea(ParseResult),
    Pmtk(PmtkResponse)
}

#[cfg(feature = "defmt")]
impl Format for GpsResponse {
    fn format(&self, fmt: defmt::Formatter) {
        match self {
            GpsResponse::Nmea(sentence) => match sentence {
                ParseResult::GGA(data) => defmt::write!(fmt, "NMEA {}", data),
                ParseResult::RMC(data) => defmt::write!(fmt, "NMEA {}", data),
                _ => defmt::write!(fmt, "NMEA {:?}", sentence),
            }
            GpsResponse::Pmtk(response) => match response {
                PmtkResponse::Ack(dt) => defmt::write!(fmt, "PMTK {:?}", dt),
                PmtkResponse::SysMsg(dt) => defmt::write!(fmt, "PMTK {:?}", dt),
                PmtkResponse::TxtMsg(dt) => defmt::write!(fmt, "PMTK {:?}", dt),
                PmtkResponse::DgpsMode(dt) => defmt::write!(fmt, "PMTK {:?}", dt),
                PmtkResponse::SbasEnabled(dt) => defmt::write!(fmt, "PMTK {:?}", dt),
                PmtkResponse::NmeaOutput(dt) => defmt::write!(fmt, "PMTK {:?}", dt),
                PmtkResponse::SbasMode(dt) => defmt::write!(fmt, "PMTK {:?}", dt),
                PmtkResponse::NavThreshold(dt) => defmt::write!(fmt, "PMTK {:?}", dt),
                PmtkResponse::Release(dt) => defmt::write!(fmt, "PMTK {:?}", dt),
                PmtkResponse::EpoInfo(dt) => defmt::write!(fmt, "PMTK {:?}", dt),
                PmtkResponse::EasyEnable(dt) => defmt::write!(fmt, "PMTK {:?}", dt),
                PmtkResponse::Log(dt) => defmt::write!(fmt, "PMTK {:?}", dt),
                PmtkResponse::Lox(dt) => defmt::write!(fmt, "PMTK {:?}", dt),
            }
        }
    }
}

/// GPS data that is a prioritized union of GGA, RMC and GLL payloads.
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Default)]
pub struct GpsReading { // TODO wrap non-primitive types?
    pub altitude: Option<f32>,
    pub faa_mode: Option<FaaMode>,
    #[cfg_attr(feature = "defmt", defmt(Debug2Format))]
    pub fix_date: Option<NaiveDate>,
    pub fix_satellites: Option<u32>,
    #[cfg_attr(feature = "defmt", defmt(Debug2Format))]
    pub fix_time: Option<NaiveTime>,
    pub fix_type: Option<FixType>,
    pub geoid_separation: Option<f32>,
    pub hdop: Option<f32>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub magnetic_variation: Option<f32>,
    pub nav_status: Option<RmcNavigationStatus>,
    pub speed_over_ground: Option<f32>,
    pub status_of_fix: Option<RmcStatusOfFix>,
    pub true_course: Option<f32>,
    pub valid: bool,
}

impl GpsReading {

    /// Constructs a new instance prioritizing GGA data over RMC data over GLL data.
    pub fn new(gga: GgaData, gll: GllData, rmc: RmcData) -> Self {
        let mut reading = Self::default();
        // priority: high -> low
        reading.apply_gga(gga);
        reading.apply_rmc(rmc);
        reading.apply_gll(gll);
        reading
    }

    fn apply_gga(&mut self, gga: GgaData) {
        self.altitude = gga.altitude;
        self.fix_satellites = gga.fix_satellites;
        self.fix_time = gga.fix_time;
        self.fix_type = gga.fix_type;
        self.geoid_separation = gga.geoid_separation;
        self.hdop = gga.hdop;
        self.latitude = gga.latitude;
        self.longitude = gga.longitude;
    }

    fn apply_gll(&mut self, gll: GllData) {
        if self.fix_time.is_none() {
            self.fix_time = gll.fix_time;
        }
        if self.latitude.is_none() {
            self.latitude = gll.latitude;
        }
        if self.longitude.is_none() {
            self.longitude = gll.longitude;
        }
        self.faa_mode = gll.faa_mode;
        self.valid = gll.valid;
    }

    fn apply_rmc(&mut self, rmc: RmcData) {
        if self.fix_time.is_none() {
            self.fix_time = rmc.fix_time;
        }
        if self.latitude.is_none() {
            self.latitude = rmc.lat;
        }
        if self.longitude.is_none() {
            self.longitude = rmc.lon;
        }
        if self.faa_mode.is_none() {
            self.faa_mode = rmc.faa_mode;
        }
        self.fix_date = rmc.fix_date;
        self.speed_over_ground = rmc.speed_over_ground;
        self.status_of_fix = Some(rmc.status_of_fix);
        self.true_course = rmc.true_course;
        self.magnetic_variation = rmc.magnetic_variation;
        self.nav_status = rmc.nav_status;
    }
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug)]
pub enum GpsError<UART> {
    Nmea, // TODO wrap nmea::Error? (would need explicit lifetime...)
    Pmtk(PmtkError),
    Uart(UART),
    Unexpected,
    Utf8,
}

impl<UART> From<Utf8Error> for GpsError<UART> {
    fn from(_: Utf8Error) -> Self {
        Self::Utf8
    }
}

impl<UART> From<PmtkError> for GpsError<UART> {
    fn from(e: PmtkError) -> Self {
        Self::Pmtk(e)
    }
}