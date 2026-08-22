use embedded_io_async::{ErrorType, Read, Write};
use nmea::Nmea;
use pmtk::traits::{Cmd, Q};
use crate::{uart, SENTENCE_MAX_LEN, RawSentence, GpsResponse, LINE_FEED};
use crate::error::GpsError;

pub struct Gps<UART> {
    buf: [u8; SENTENCE_MAX_LEN],
    buf_idx: usize,
    nmea: Nmea,
    uart: UART,
}

impl<UART: Read + Write + ErrorType> Gps<UART> {
    pub fn new(uart: UART) -> Self {
        Self {
            buf: [0u8; SENTENCE_MAX_LEN],
            buf_idx: 0,
            nmea: Nmea::default(),
            uart
        }
    }

    /// Sends a PMTK command.
    pub async fn command(&mut self, command: impl Cmd) -> Result<(), GpsError<UART::Error>> {
        uart::write(&mut self.uart, command.serialize()?.as_bytes()).await
    }

    /// Parses a raw sentence into a NMEA sentence.
    pub async fn parse_nmea_sentence(&mut self, sentence: &RawSentence) -> Result<GpsResponse, GpsError<UART::Error>> {
        uart::parse_nmea_sentence::<UART>(&mut self.nmea, sentence).await
    }

    /// Parses a raw sentence into a PMTK sentence.
    pub async fn parse_pmtk_sentence(&mut self, sentence: &RawSentence) -> Result<GpsResponse, GpsError<UART::Error>> {
        uart::parse_pmtk_sentence::<UART>(sentence).await
    }

    /// Parses a raw sentence into a NMEA or PMTK sentence.
    pub async fn parse_sentence(&mut self, sentence: &RawSentence) -> Result<GpsResponse, GpsError<UART::Error>> {
        uart::parse_sentence::<UART>(&mut self.nmea, sentence).await
    }

    /// Sends a PMTK query.
    pub async fn query(&mut self, query: impl Q) -> Result<(), GpsError<UART::Error>> {
        uart::write(&mut self.uart, query.serialize()?.as_bytes()).await
    }

    // reads a sentence
    pub async fn read_sentence(&mut self) -> Result<Option<RawSentence>, GpsError<UART::Error>> {
        uart::read_sentence(&mut self.uart, &mut self.buf, &mut self.buf_idx).await
    }

    fn reset_buffer(&mut self) {
        self.buf = [0; 255];
        self.buf_idx = 0;
    }
}