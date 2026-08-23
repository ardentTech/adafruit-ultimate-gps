use embedded_io_async::{ErrorType, Read, Write};
use pmtk::traits::{Cmd, Q};
use crate::error::GpsError;
use crate::GpsResponse;
use crate::gps::{GpsReader, GpsWriter};

pub struct GpsRx<UART> {
    reader: GpsReader,
    uart: UART,
}

impl<UART: Read + ErrorType> GpsRx<UART> {
    pub fn new(uart: UART) -> Self {
        Self {
            reader: GpsReader::default(),
            uart,
        }
    }

    /// Reads a NMEA or PMTK response.
    pub async fn read_response(&mut self) -> Result<Option<GpsResponse>, GpsError<UART::Error>> {
        self.reader.read_response(&mut self.uart).await
    }
}

pub struct GpsTx<UART> {
    uart: UART,
    writer: GpsWriter,
}

impl<UART: Write + ErrorType> GpsTx<UART> {
    pub fn new(uart: UART) -> Self {
        Self { uart, writer: GpsWriter {} }
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