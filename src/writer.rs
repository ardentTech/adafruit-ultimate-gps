use crate::types::GpsError;
#[cfg(feature = "defmt")]
use defmt::debug;
use embedded_io_async::{ErrorType, Write};
use pmtk::traits::CmdQ;

pub struct GpsWriter {}

impl GpsWriter {

    /// Writes a PMTK command or query to the UART.
    pub async fn send<UART: Write + ErrorType>(&mut self, uart: &mut UART, command: impl CmdQ) -> Result<(), GpsError<UART::Error>> {
        #[cfg(feature = "defmt")]
        debug!("GpsWriter.send: {}", command.serialize()?.as_bytes());
        Ok(uart.write_all(command.serialize()?.as_bytes()).await.map_err(GpsError::Uart)?)
    }
}