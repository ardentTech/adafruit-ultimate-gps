#![no_std]

mod error;
mod reader;
mod writer;

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

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, PartialEq)]
pub enum GpsResponse {
    Nmea(SentenceType),
    Pmtk(PmtkResponse)
}

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

/// TODO
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

/// TODO
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
        // TODO could wait for ack
        self.tx.command(&mut self.uart, command).await
    }

    pub async fn query(&mut self, query: impl Q) -> Result<(), GpsError<UART::Error>> {
        // TODO could wait for dt
        self.tx.query(&mut self.uart, query).await
    }

    pub async fn read(&mut self) -> Result<Option<GpsResponse>, GpsError<UART::Error>> {
        self.rx.read_response(&mut self.uart).await
    }
}