#[cfg(feature = "defmt")]
use defmt::debug;
use embedded_io_async::{ErrorType, Write};
use pmtk::traits::CmdQ;
use crate::error::GpsError;

pub(crate) struct GpsWriter {}

impl GpsWriter {
    pub(crate) async fn send<UART: Write + ErrorType>(&mut self, uart: &mut UART, command: impl CmdQ) -> Result<(), GpsError<UART::Error>> {
        self.write_to_uart(uart, command.serialize()?.as_bytes()).await
    }

    async fn write_to_uart<UART: Write + ErrorType>(&mut self, uart: &mut UART, buf: &[u8]) -> Result<(), GpsError<UART::Error>> {
        #[cfg(feature = "defmt")]
        debug!("GpsWriter.write_to_uart: {}", buf);
        Ok(uart.write_all(buf).await.map_err(GpsError::Uart)?)
    }
}