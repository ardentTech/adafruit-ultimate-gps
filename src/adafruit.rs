use chrono::NaiveTime;
use embedded_io_async::{ErrorType, Read, Write};
use nmea::ParseResult;
use nmea::sentences::FixType;
use pmtk::dt::nmea_output::Frequency;
use crate::adafruit::Source::{GGA, GLL, RMC};
use crate::error::GpsError;
use crate::reader::GpsReader;
use crate::types::GpsResponse;
use crate::writer::GpsWriter;

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Source {
    GGA,
    GLL,
    RMC
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Payload {
    #[cfg_attr(feature = "defmt", defmt(Debug2Format))]
    pub fix_time: Option<NaiveTime>,
    pub fix_type: Option<FixType>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub source: Source,
    // fix_satellites
    // hdop
    // altitude
    // geoid
}

// TODO this has to be half-duplex yeah?
// TODO state machine?
pub struct Adafruit<UART> {
    rx: GpsReader,
    tx: GpsWriter,
    uart: UART,
}

// GGA (GLL backup) + RMC
impl<UART: Read + Write + ErrorType> Adafruit<UART> {

    pub fn new(uart: UART) -> Self {
        Self { rx: GpsReader::default(), tx: GpsWriter {}, uart }
    }

    // TODO should this just return something and not update state?
    // Adafruit_GPS::parse checks for GGA and updates state
    pub async fn read_sentence(&mut self) -> Result<Payload, GpsError<UART::Error>> {
        let mut payload: Option<Payload> = None;

        while payload.is_none() {
            match self.rx.read_response(&mut self.uart).await? {
                Some(gps_res) => match gps_res {
                    GpsResponse::Nmea(nmea_res) => match nmea_res {
                        ParseResult::GGA(data) => {
                            payload = Some(Payload {
                                fix_time: data.fix_time,
                                fix_type: data.fix_type,
                                latitude: data.latitude,
                                longitude: data.longitude,
                                source: GGA,
                            })
                        }
                        ParseResult::GLL(data) => {
                            payload = Some(Payload {
                                fix_time: data.fix_time,
                                fix_type: None,
                                latitude: data.latitude,
                                longitude: data.longitude,
                                source: GLL,
                            })
                        }
                        ParseResult::RMC(data) => {
                            payload = Some(Payload {
                                fix_time: data.fix_time,
                                fix_type: None,
                                latitude: data.lat,
                                longitude: data.lon,
                                source: RMC
                            })
                        }
                        _ => {}
                    }
                    _ => {}
                }
                _ => {}
            }
        }
        Ok(payload.ok_or(GpsError::Unexpected)?)
    }

    // configures the chip for GGA, GLL and RMC
    pub async fn start(&mut self, frequency_ms: u16) -> Result<(), GpsError<UART::Error>> {
        self.tx.send(&mut self.uart, pmtk::cmd::set_nmea_output::SetNmeaOutputCmd::new(
            Frequency::OnceEveryFivePositionFixes,
            Frequency::OnceEveryFivePositionFixes,
            Frequency::Disabled,
            Frequency::OnceEveryFivePositionFixes,
            Frequency::Disabled,
            Frequency::Disabled,
            Frequency::Disabled,
        )).await?;

        self.tx.send(
            &mut self.uart,
            pmtk::cmd::set_nmea_update_rate::SetNmeaUpdateRateCmd::new(frequency_ms)?
        ).await
    }
}