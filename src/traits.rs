use embedded_io_async::{ErrorType, Read, Write};
use pmtk::traits::{Cmd, Q};
use crate::error::GpsError;
use crate::GpsResponse;

pub trait GpsRead<UART: Read + ErrorType> {
    async fn read_response(&mut self) -> Result<Option<GpsResponse>, GpsError<UART::Error>>;
}

pub trait GpsWrite<UART: Write + ErrorType> {
    async fn send_command(&mut self, command: impl Cmd) -> Result<(), GpsError<UART::Error>>;
    async fn send_query(&mut self, query: impl Q) -> Result<(), GpsError<UART::Error>>;
}