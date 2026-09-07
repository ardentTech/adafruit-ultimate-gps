use chrono::{NaiveDate, NaiveTime};
use crate::error::GpsError;
use crate::types::SENTENCE_MAX_LEN;
use crate::types::{GpsResponse, RawSentence};
#[cfg(feature = "defmt")]
use defmt::{debug, error, info};
use embedded_io_async::{ErrorType, Read};
use heapless::Vec;
use nmea::parse_str;
use nmea::sentences::rmc::{RmcNavigationStatus, RmcStatusOfFix};
use nmea::sentences::{FaaMode, FixType, GgaData, GllData, RmcData};
use pmtk::response::PmtkResponse;

const LINE_FEED: u8 = 0x0a; // '\n'

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
    pub fix_type: Option<FixType>, // TODO wrap this?
    pub geoid_separation: Option<f32>,
    pub hdop: Option<f32>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub magnetic_variation: Option<f32>,
    pub nav_status: Option<RmcNavigationStatus>, // TODO wrap this?
    pub speed_over_ground: Option<f32>,
    pub status_of_fix: RmcStatusOfFix, // TODO wrap this?
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

pub(crate) struct GpsReader {
    buffer: [u8; SENTENCE_MAX_LEN],
    buffer_idx: usize,
}

impl Default for GpsReader {
    fn default() -> Self {
        Self { buffer: [0u8; SENTENCE_MAX_LEN], buffer_idx: 0 }
    }
}

impl GpsReader {
    pub(crate) async fn parse<UART: Read + ErrorType>(&mut self, sentence: &RawSentence) -> Result<GpsResponse, GpsError<UART::Error>> {
        #[cfg(feature = "defmt")]
        debug!("GpsReader.parse()");
        match self.parse_nmea::<UART>(sentence).await {
            Ok(res) => Ok(res),
            Err(e) => {
                match e {
                    // if nmea couldn't parse the sentence, try pmtk
                    nmea::Error::ParsingError(_) => match self.parse_pmtk::<UART>(sentence).await {
                        Ok(res) => Ok(res),
                        Err(e) => {
                            Err(e)
                        },
                    },
                    _ => Err(GpsError::Nmea)
                }
            }
        }
    }

    async fn parse_nmea<'a, UART: Read + ErrorType>(&mut self, sentence: &'a RawSentence) -> Result<GpsResponse, nmea::Error<'a>> {
        #[cfg(feature = "defmt")]
        debug!("GpsReader.parse_nmea()");
        Ok(GpsResponse::Nmea(parse_str(sentence)?))
    }

    async fn parse_pmtk<UART: Read + ErrorType>(&mut self, sentence: &RawSentence) -> Result<GpsResponse, GpsError<UART::Error>> {
        #[cfg(feature = "defmt")]
        debug!("GpsReader.parse_pmtk()");
        Ok(GpsResponse::Pmtk(PmtkResponse::try_from(sentence.as_bytes())?))
    }

    pub(crate) async fn read_sentence<UART: Read + ErrorType>(&mut self, uart: &mut UART) -> Result<Option<RawSentence>, GpsError<UART::Error>> {
        #[cfg(feature = "defmt")]
        debug!("GpsReader.read_sentence()");
        let mut buf = [0u8; SENTENCE_MAX_LEN];

        match uart.read(&mut buf).await.map_err(GpsError::Uart) {
            Ok(len) => {
                let mut res = Ok(None);
                if len > 0 {
                    buf[..len].iter().for_each(|b| {
                        self.buffer[self.buffer_idx] = *b;

                        if *b == LINE_FEED {
                            #[cfg(feature = "defmt")]
                            debug!("raw sentence: {}", &self.buffer[..=self.buffer_idx]);
                            match <Vec<u8, SENTENCE_MAX_LEN>>::try_from(&self.buffer[..=self.buffer_idx]) {
                                Ok(v) => {
                                    res = match RawSentence::from_utf8(v) {
                                        Ok(raw) => Ok(Some(raw)),
                                        Err(_) => Err(GpsError::Utf8)
                                    };
                                    self.reset_buffer();
                                }
                                Err(_) => res = Err(GpsError::Unexpected)
                            }
                        } else {
                            if self.buffer_idx + 1 == SENTENCE_MAX_LEN {
                                self.reset_buffer();
                            } else {
                                self.buffer_idx += 1;
                            }
                        }
                    });
                }
                res
            }
            Err(e) => Err(e)
        }
    }

    fn reset_buffer(&mut self) {
        #[cfg(feature = "defmt")]
        debug!("GpsReader.reset_buffer()");
        self.buffer = [0; SENTENCE_MAX_LEN];
        self.buffer_idx = 0;
    }
}