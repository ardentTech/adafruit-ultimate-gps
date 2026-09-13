use crate::types::GpsError;
#[cfg(feature = "defmt")]
use defmt::debug;
use embedded_io_async::{ErrorType, Write};
use pmtk::traits::CmdQ;

pub struct GpsWriter {}

impl GpsWriter {

    /// Writes a PMTK command or query to the UART.
    ///
    /// * `uart` - the UART driver to write to.
    /// * `cmd_q` - the PMTK command or query to write to the UART.
    pub async fn send<UART: Write + ErrorType>(&mut self, uart: &mut UART, cmd_q: impl CmdQ) -> Result<(), GpsError<UART::Error>> {
        #[cfg(feature = "defmt")]
        debug!("GpsWriter.send: {}", cmd_q.serialize()?.as_bytes());
        Ok(uart.write_all(cmd_q.serialize()?.as_bytes()).await.map_err(GpsError::Uart)?)
    }
}