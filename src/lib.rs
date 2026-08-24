#![no_std]

mod error;
mod reader;
mod writer;

use defmt::Format;
use embedded_io_async::{ErrorType, Read, Write};
use heapless::String;
use nmea::SentenceType;

pub use pmtk;
use pmtk::response::PmtkResponse;
use pmtk::traits::{Cmd, Q};
use crate::error::GpsError;
use crate::reader::GpsReader;
use crate::writer::GpsWriter;

const LINE_FEED: u8 = 0x0a; // '\n'
const SENTENCE_MAX_LEN: usize = 255;

pub type RawSentence = String<SENTENCE_MAX_LEN>;

#[derive(Debug, Format)]
pub enum GpsResponse {
    Nmea(SentenceType),
    Pmtk(PmtkResponse)
}

// ----------------------------------------------------------------------------

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
}

// ----------------------------------------------------------------------------

pub struct GpsTx<UART> {
    uart: UART,
    tx: GpsWriter,
}
impl<UART: Write + ErrorType> GpsTx<UART> {
    pub fn new(uart: UART) -> Self {
        Self { uart, tx: GpsWriter {} }
    }

    pub async fn command(&mut self, command: impl Cmd) -> Result<(), GpsError<UART::Error>> {
        self.tx.command(&mut self.uart, command).await
    }

    pub async fn query(&mut self, query: impl Q) -> Result<(), GpsError<UART::Error>> {
        self.tx.query(&mut self.uart, query).await
    }
}

// ----------------------------------------------------------------------------

pub struct Gps<UART> {
    rx: GpsReader,
    tx: GpsWriter,
    uart: UART,
}
impl<UART: Read + Write + ErrorType> Gps<UART> {

    pub fn new(uart: UART) -> Self {
        Self { rx: GpsReader::default(), tx: GpsWriter {}, uart }
    }

    pub async fn command(&mut self, command: impl Cmd) -> Result<(), GpsError<UART::Error>> {
        self.tx.command(&mut self.uart, command).await
    }

    pub async fn query(&mut self, query: impl Q) -> Result<(), GpsError<UART::Error>> {
        self.tx.query(&mut self.uart, query).await
    }

    pub async fn read(&mut self) -> Result<Option<GpsResponse>, GpsError<UART::Error>> {
        self.rx.read_response(&mut self.uart).await
    }
}