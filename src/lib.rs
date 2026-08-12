#![no_std]

mod error;

use defmt::Format;
use embedded_io_async::{ErrorType, Read, Write};
use heapless::{String, Vec};
use nmea::{Nmea, SentenceType};
use pmtk::traits::{Cmd, Q};
use pmtk::response::PmtkResponse;
use crate::error::GpsError;
use crate::error::GpsError::UnexpectedNumBytes;

pub use pmtk;
use pmtk::cmd::aic_mode::AicModeCmd;
use pmtk::q::datum::DatumQ;

const LINE_FEED: u8 = 0x0a; // '\n'
const SENTENCE_MAX_LEN: usize = 255;

pub type RawSentence = String<SENTENCE_MAX_LEN>;

#[derive(Debug)]
pub enum GpsRequest {
    AicMode(AicModeCmd),
    DatumQuery(DatumQ),
}

#[derive(Debug, Format)]
pub enum GpsResponse {
    Nmea(SentenceType),
    Pmtk(PmtkResponse)
}

pub struct Gps<UART> {
    buf: [u8; SENTENCE_MAX_LEN],
    buf_idx: usize,
    nmea: Nmea,
    // TODO this is needed for simplex or half duplex only
    uart: UART,
    // TODO full duplex would need BufferedUartRx and BufferedUartTx
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
        self.write(command.serialize()?.as_bytes()).await
    }

    /// Parses a raw sentence into a NMEA sentence.
    pub async fn parse_nmea_sentence(&mut self, sentence: &RawSentence) -> Result<GpsResponse, GpsError<UART::Error>> {
        Ok(GpsResponse::Nmea(self.nmea.parse(sentence).map_err(|_| GpsError::Nmea)?))
    }

    /// Parses a raw sentence into a PMTK sentence.
    pub async fn parse_pmtk_sentence(&mut self, sentence: &RawSentence) -> Result<GpsResponse, GpsError<UART::Error>> {
        Ok(GpsResponse::Pmtk(PmtkResponse::try_from(sentence.as_bytes())?))
    }

    /// Parses a raw sentence into a NMEA or PMTK sentence.
    pub async fn parse_sentence(&mut self, sentence: &RawSentence) -> Result<GpsResponse, GpsError<UART::Error>> {
        match self.parse_nmea_sentence(sentence).await {
            Ok(res) => Ok(res),
            Err(_) => self.parse_pmtk_sentence(sentence).await,
        }
    }

    /// Sends a PMTK query.
    pub async fn query(&mut self, query: impl Q) -> Result<(), GpsError<UART::Error>> {
        self.write(query.serialize()?.as_bytes()).await
    }

    // reads a sentence
    pub async fn read_sentence(&mut self) -> Result<Option<RawSentence>, GpsError<UART::Error>> {
        let mut buf = [0u8; SENTENCE_MAX_LEN];

        match self.read(&mut buf).await {
            Ok(len) => {
                let mut res = Ok(None);
                if len > 0 {
                    buf[..len].iter().for_each(|b| {
                        self.buf[self.buf_idx] = *b; // copy ASCII char

                        if *b == LINE_FEED {
                            let v = <Vec<u8, SENTENCE_MAX_LEN>>::try_from(&self.buf[..=self.buf_idx]).unwrap(); // TODO remove unwrap()
                            res = Ok(Some(RawSentence::from_utf8(v).unwrap())); // TODO remove unwrap()
                            self.reset_buffer();
                        } else {
                            if self.buf_idx + 1 == SENTENCE_MAX_LEN {
                                self.reset_buffer();
                            } else {
                                self.buf_idx += 1;
                            }
                        }
                    });
                }
                res
            }
            Err(e) => Err(e)
        }
    }

    // reads directly from UART
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, GpsError<UART::Error>> {
        self.uart.read(buf).await.map_err(GpsError::Uart)
    }

    fn reset_buffer(&mut self) {
        self.buf = [0; 255];
        self.buf_idx = 0;
    }

    // writes directly to UART
    async fn write(&mut self, data: &[u8]) -> Result<(), GpsError<UART::Error>> {
        if data.len() != self.uart.write(data).await.map_err(GpsError::Uart)? {
            return Err(UnexpectedNumBytes) // TODO not happy with the name of this error
        }
        Ok(())
    }
}