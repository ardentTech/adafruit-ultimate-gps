use embedded_io_async::{ErrorType, Write};
use pmtk::traits::{Cmd, Q};
use crate::error::GpsError;
use crate::error::GpsError::UnexpectedNumBytes;

pub(crate) struct GpsWriter {}
impl GpsWriter {
    pub(crate) async fn command<UART: Write + ErrorType>(&mut self, uart: &mut UART, command: impl Cmd) -> Result<(), GpsError<UART::Error>> {
        self.uart_write(uart, command.serialize()?.as_bytes()).await
    }

    pub(crate) async fn query<UART: Write + ErrorType>(&mut self, uart: &mut UART, query: impl Q) -> Result<(), GpsError<UART::Error>> {
        self.uart_write(uart, query.serialize()?.as_bytes()).await
    }

    async fn uart_write<UART: Write + ErrorType>(&mut self, uart: &mut UART, buf: &[u8]) -> Result<(), GpsError<UART::Error>> {
        if buf.len() != uart.write(buf).await.map_err(GpsError::Uart)? {
            return Err(UnexpectedNumBytes) // TODO not happy with the name of this error
        }
        Ok(())
    }
}