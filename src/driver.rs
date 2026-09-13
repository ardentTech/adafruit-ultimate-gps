use crate::reader::GpsReader;
use crate::types::GpsError;
use crate::types::{GpsReading, GpsResponse, RawSentence};
use crate::writer::GpsWriter;
#[cfg(feature = "defmt")]
use defmt::debug;
use embedded_io_async::{ErrorType, Read, Write};
use nmea::ParseResult;
use nmea::sentences::{GgaData, GllData, RmcData};
use pmtk::dt::ack::AckFlag;
use pmtk::dt::nmea_output::Frequency;
use pmtk::response::PmtkResponse;
use pmtk::traits::{CmdQ, Packet};

// TODO locus logger

pub struct AdafruitUltimateGps<UART> {
    rx: GpsReader,
    tx: GpsWriter,
    uart: UART,
}

// GGA (GLL backup) + RMC
impl<UART: Read + Write + ErrorType> AdafruitUltimateGps<UART> {

    /// Constructs a new driver instance.
    pub fn new(uart: UART) -> Self {
        #[cfg(feature = "defmt")]
        debug!("AdafruitUltimateGps.new()");
        Self { rx: GpsReader::default(), tx: GpsWriter {}, uart }
    }

    /// Reads Gps data.
    pub async fn read(&mut self) -> Result<GpsReading, GpsError<UART::Error>> {
        #[cfg(feature = "defmt")]
        debug!("AdafruitUltimateGps.read()");
        let mut gga: Option<GgaData> = None;
        let mut gll: Option<GllData> = None;
        let mut rmc: Option<RmcData> = None;

        while gga.is_none() | gll.is_none() | rmc.is_none() {
            match self.rx.read_response(&mut self.uart).await? {
                Some(gps_res) => match gps_res {
                    GpsResponse::Nmea(nmea_res) => match nmea_res {
                        ParseResult::GGA(data) => gga = Some(data),
                        ParseResult::GLL(data) => gll = Some(data),
                        ParseResult::RMC(data) => rmc = Some(data),
                        _ => {} // other NMEA responses nop
                    }
                    _ => {} // PMTK responses nop
                }
                _ => {} // nop as errors are propagated above
            }
        }
        Ok(GpsReading::new(gga.unwrap(), gll.unwrap(), rmc.unwrap()))
    }

    /// Reads a raw NMEA or PMTK sentence.
    pub async fn read_sentence(&mut self) -> Result<Option<RawSentence>, GpsError<UART::Error>> {
        #[cfg(feature = "defmt")]
        debug!("AdafruitUltimateGps.read_sentence()");
        self.rx.read_sentence(&mut self.uart).await
    }

    /// Sends a PMTK command with reception verification.
    ///
    /// * `cmd_q` - the PMTK command or query to write to the UART.
    /// * `max_attempts` - the maximum number of responses to read while attempting to verify reception.
    pub async fn send<T: CmdQ>(&mut self, cmd_q: T, max_attempts: u8) -> Result<bool, GpsError<UART::Error>> {
        #[cfg(feature = "defmt")]
        debug!("AdafruitUltimateGps.send()");
        self.tx.send(&mut self.uart, cmd_q).await?;
        let mut i = 0;
        let mut verified = false;

        while i < max_attempts {
            if let Some(gps_res) = self.rx.read_response(&mut self.uart).await? {
                match gps_res {
                    GpsResponse::Pmtk(pmtk_res) => match pmtk_res {
                        PmtkResponse::Ack(dt) => {
                            if dt.cmd == <T as Packet>::PKT_TYPE && dt.flag == AckFlag::ActionSucceeded {
                                verified = true;
                                break;
                            }
                        }
                        _ => {}
                    }
                    _ => {}
                }
                i += 1;
            }
        }

        Ok(verified)
    }

    /// Sends a PMTK command without reception verification.
    ///
    /// * `cmd_q` - the PMTK command or query to write to the UART.
    pub async fn send_unverified(&mut self, cmd_q: impl CmdQ) -> Result<(), GpsError<UART::Error>> {
        #[cfg(feature = "defmt")]
        debug!("AdafruitUltimateGps.send_unverified()");
        self.tx.send(&mut self.uart, cmd_q).await
    }

    /// Configures the chip for reading Gps data.
    ///
    /// * `update_rate_ms` - the millisecond frequency for NMEA updates.
    pub async fn start(&mut self, update_rate_ms: u16) -> Result<(), GpsError<UART::Error>> {
        #[cfg(feature = "defmt")]
        debug!("AdafruitUltimateGps.start()");
        self.tx.send(&mut self.uart, pmtk::cmd::set_nmea_output::SetNmeaOutputCmd::new(
            // TODO change pmtk lib bc unnamed params here suck
            Frequency::OnceEveryOnePositionFix,
            Frequency::OnceEveryOnePositionFix,
            Frequency::Disabled,
            Frequency::OnceEveryOnePositionFix,
            Frequency::Disabled,
            Frequency::Disabled,
            Frequency::Disabled,
        )).await?;

        self.tx.send(
            &mut self.uart,
            pmtk::cmd::set_nmea_update_rate::SetNmeaUpdateRateCmd::new(update_rate_ms)?
        ).await
    }
}