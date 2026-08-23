use embedded_io_async::{ErrorType, Read, Write};
use pmtk::traits::{Cmd, Q};
use crate::error::GpsError;
use crate::gps::{GpsReader, GpsWriter};
use crate::GpsResponse;

pub struct Gps<UART> {
    reader: GpsReader,
    uart: UART,
    writer: GpsWriter,
}

impl<UART: Read + Write + ErrorType> Gps<UART> {
    pub fn new(uart: UART) -> Self {
        Self {
            reader: GpsReader::default(),
            uart,
            writer: GpsWriter {}
        }
    }

    /// Reads a NMEA or PMTK response.
    pub async fn read_response(&mut self) -> Result<Option<GpsResponse>, GpsError<UART::Error>> {
        self.reader.read_response(&mut self.uart).await
    }

    /// Sends a PMTK command.
    pub async fn send_command(&mut self, command: impl Cmd) -> Result<(), GpsError<UART::Error>> {
        self.writer.send_command(&mut self.uart, command).await
    }

    /// Sends a PMTK query.
    pub async fn send_query(&mut self, query: impl Q) -> Result<(), GpsError<UART::Error>> {
        self.writer.send_query(&mut self.uart, query).await
    }
}