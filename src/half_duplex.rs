use embedded_io_async::{ErrorType, Read, Write};
use nmea::Nmea;
use pmtk::traits::{Cmd, Q};
use crate::error::GpsError;
use crate::gps::GpsReader;
use crate::{uart, GpsResponse};

pub struct Gps<UART> {
    reader: GpsReader,
    nmea: Nmea,
    uart: UART,
}

impl<UART: Read + Write + ErrorType> Gps<UART> {
    pub fn new(uart: UART) -> Self {
        Self {
            reader: GpsReader::default(),
            nmea: Nmea::default(),
            uart
        }
    }

    /// Reads a NMEA or PMTK response.
    pub async fn read_response(&mut self) -> Result<Option<GpsResponse>, GpsError<UART::Error>> {
        match self.reader.read_sentence(&mut self.uart).await {
            Ok(res) => if let Some(raw) = res {
                match self.reader.parse_sentence::<UART>(&mut self.nmea, &raw).await {
                    Ok(res) => Ok(Some(res)),
                    Err(e) => Err(e)
                }
            } else { Ok(None) },
            Err(e) => Err(e)
        }
    }

    /// Sends a PMTK command.
    pub async fn send_command(&mut self, command: impl Cmd) -> Result<(), GpsError<UART::Error>> {
        uart::write(&mut self.uart, command.serialize()?.as_bytes()).await
    }

    /// Sends a PMTK query.
    pub async fn send_query(&mut self, query: impl Q) -> Result<(), GpsError<UART::Error>> {
        uart::write(&mut self.uart, query.serialize()?.as_bytes()).await
    }
}