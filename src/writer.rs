#[cfg(feature = "defmt")]
use defmt::debug;
use embedded_io_async::{ErrorType, Write};
use pmtk::traits::{Cmd, Q};
use crate::error::GpsError;

pub(crate) struct GpsWriter {}

impl GpsWriter {
    pub(crate) async fn command<UART: Write + ErrorType>(&mut self, uart: &mut UART, command: impl Cmd) -> Result<(), GpsError<UART::Error>> {
        self.write_to_uart(uart, command.serialize()?.as_bytes()).await
    }

    pub(crate) async fn query<UART: Write + ErrorType>(&mut self, uart: &mut UART, query: impl Q) -> Result<(), GpsError<UART::Error>> {
        self.write_to_uart(uart, query.serialize()?.as_bytes()).await
    }

    async fn write_to_uart<UART: Write + ErrorType>(&mut self, uart: &mut UART, buf: &[u8]) -> Result<(), GpsError<UART::Error>> {
        #[cfg(feature = "defmt")]
        debug!("GpsWriter.write_to_uart: {}", buf);
        Ok(uart.write_all(buf).await.map_err(GpsError::Uart)?)
    }
}