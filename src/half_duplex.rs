use crate::error::GpsError;
use crate::reader::GpsReader;
use crate::types::{GpsResponse, RawSentence};
use crate::writer::GpsWriter;
use embedded_io_async::{ErrorType, Read, Write};
use pmtk::cmd::locus_stop_logger::LocusStopLogger;
use pmtk::dt::ack::AckFlag;
use pmtk::q::locus_data::LocusDataQ;
use pmtk::response::PmtkResponse;
use pmtk::traits::{CmdQ, Packet};

/// Half-duplex Adafruit Ultimate GPS driver.
pub struct Gps<UART> {
    rx: GpsReader,
    tx: GpsWriter,
    uart: UART,
}
impl<UART: Read + Write + ErrorType> Gps<UART> {
    pub fn new(uart: UART) -> Self {
        Self { rx: GpsReader::default(), tx: GpsWriter {}, uart }
    }

    pub async fn read(&mut self) -> Result<Option<GpsResponse>, GpsError<UART::Error>> {
        self.rx.read_response(&mut self.uart).await
    }

    pub async fn read_raw(&mut self) -> Result<Option<RawSentence>, GpsError<UART::Error>> {
        self.rx.read_sentence(&mut self.uart).await
    }

    pub async fn send(&mut self, request: impl CmdQ) -> Result<(), GpsError<UART::Error>> {
        self.tx.send(&mut self.uart, request).await
    }

    pub async fn send_and_verify<T: CmdQ>(&mut self, request: T, max_attempts: u8) -> Result<bool, GpsError<UART::Error>> {
        self.send(request).await?;
        let mut i = 0;
        let mut verified = false;

        while i < max_attempts {
            if let Some(gps_res) = self.read().await? {
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

    pub async fn start_logger(&mut self) -> Result<(), GpsError<UART::Error>> {
        self.send(LocusStopLogger::new(false)).await
    }

    pub async fn query_logger(&mut self) -> Result<(), GpsError<UART::Error>> {
        self.send(LocusDataQ::new(false)).await
    }
}