use crate::error::GpsError;
use crate::reader::GpsReader;
use crate::types::{GpsResponse, RawSentence};
use crate::writer::GpsWriter;
use embedded_io_async::{ErrorType, Read, Write};
use pmtk::traits::CmdQ;

/// TODO
pub struct GpsRx<UART> {
    rx: GpsReader,
    uart: UART,
}
impl<UART: Read + ErrorType> GpsRx<UART> {
    pub fn new(uart: UART) -> Self {
        Self { rx: GpsReader::default(), uart }
    }

    pub async fn read(&mut self) -> Result<Option<GpsResponse>, GpsError<UART::Error>> {
        self.rx.read_response(&mut self.uart).await
    }

    pub async fn read_raw(&mut self) -> Result<Option<RawSentence>, GpsError<UART::Error>> {
        self.rx.read_sentence(&mut self.uart).await
    }
}

// ----------------------------------------------------------------------------



/// TODO
pub struct GpsTx<UART> {
    tx: GpsWriter,
    uart: UART,
}
impl<UART: Write + ErrorType> GpsTx<UART> {
    pub fn new(uart: UART) -> Self {
        Self { uart, tx: GpsWriter {} }
    }

    pub async fn send(&mut self, command: impl CmdQ) -> Result<(), GpsError<UART::Error>> {
        self.tx.send(&mut self.uart, command).await
    }
}