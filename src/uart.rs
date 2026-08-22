use embedded_io_async::{ErrorType, Read, Write};
use crate::error::GpsError;
use crate::error::GpsError::UnexpectedNumBytes;

pub(crate) async fn read<UART: Read + ErrorType>(uart: &mut UART, buf: &mut [u8]) -> Result<usize, GpsError<UART::Error>> {
    uart.read(buf).await.map_err(GpsError::Uart)
}

pub(crate) async fn write<UART: Write + ErrorType>(uart: &mut UART, buf: &[u8]) -> Result<(), GpsError<UART::Error>> {
    if buf.len() != uart.write(buf).await.map_err(GpsError::Uart)? {
        return Err(UnexpectedNumBytes) // TODO not happy with the name of this error
    }
    Ok(())
}