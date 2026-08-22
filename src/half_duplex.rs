use embedded_io_async::{ErrorType, Read, Write};
use heapless::Vec;
use nmea::Nmea;
use pmtk::response::PmtkResponse;
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
        uart::write(&mut self.uart, query.serialize()?.as_bytes()).await
    }

    // reads a sentence
    pub async fn read_sentence(&mut self) -> Result<Option<RawSentence>, GpsError<UART::Error>> {
        let mut buf = [0u8; SENTENCE_MAX_LEN];

        match uart::read(&mut self.uart, &mut buf).await {
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

    fn reset_buffer(&mut self) {
        self.buf = [0; 255];
        self.buf_idx = 0;
    }
}