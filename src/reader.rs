use crate::types::GpsError;
use crate::types::SENTENCE_MAX_LEN;
use crate::types::{GpsResponse, RawSentence};
#[cfg(feature = "defmt")]
use defmt::debug;
use embedded_io_async::{ErrorType, Read};
use heapless::Vec;
use nmea::parse_str;
use pmtk::response::PmtkResponse;

const LINE_FEED: u8 = 0x0a; // '\n'

/// Reads and parses GPS sentences.
pub struct GpsReader {
    buffer: [u8; SENTENCE_MAX_LEN],
    buffer_idx: usize,
}

impl Default for GpsReader {
    fn default() -> Self {
        Self { buffer: [0u8; SENTENCE_MAX_LEN], buffer_idx: 0 }
    }
}

impl GpsReader {

    /// Parses a sentence into a NMEA or, if necessary, a PMTK response.
    ///
    /// * `sentence` - the raw sentence to parse.
    pub async fn parse<UART: Read + ErrorType>(&mut self, sentence: &RawSentence) -> Result<GpsResponse, GpsError<UART::Error>> {
        #[cfg(feature = "defmt")]
        debug!("GpsReader.parse()");
        match self.parse_nmea::<UART>(sentence).await {
            Ok(res) => Ok(res),
            Err(e) => {
                match e {
                    // if nmea couldn't parse the sentence, try pmtk
                    nmea::Error::ParsingError(_) => match self.parse_pmtk::<UART>(sentence).await {
                        Ok(res) => Ok(res),
                        Err(e) => {
                            Err(e)
                        },
                    },
                    _ => Err(GpsError::Nmea)
                }
            }
        }
    }

    async fn parse_nmea<'a, UART: Read + ErrorType>(&mut self, sentence: &'a RawSentence) -> Result<GpsResponse, nmea::Error<'a>> {
        #[cfg(feature = "defmt")]
        debug!("GpsReader.parse_nmea()");
        Ok(GpsResponse::Nmea(parse_str(sentence)?))
    }

    async fn parse_pmtk<UART: Read + ErrorType>(&mut self, sentence: &RawSentence) -> Result<GpsResponse, GpsError<UART::Error>> {
        #[cfg(feature = "defmt")]
        debug!("GpsReader.parse_pmtk()");
        Ok(GpsResponse::Pmtk(PmtkResponse::try_from(sentence.as_bytes())?))
    }

    /// Reads a sentence and parses it into a response.
    ///
    /// * `uart` - the UART driver to read from.
    pub async fn read_response<UART: Read + ErrorType>(&mut self, uart: &mut UART) -> Result<Option<GpsResponse>, GpsError<UART::Error>> {
        #[cfg(feature = "defmt")]
        debug!("GpsReader.read_response()");
        match self.read_sentence(uart).await {
            Ok(res) => if let Some(raw) = res {
                match self.parse::<UART>(&raw).await {
                    Ok(res) => Ok(Some(res)),
                    Err(e) => Err(e)
                }
            } else { Ok(None) },
            Err(e) => Err(e)
        }
    }

    /// Reads a raw sentence from the UART.
    ///
    /// * `uart` - the UART driver to read from.
    pub async fn read_sentence<UART: Read + ErrorType>(&mut self, uart: &mut UART) -> Result<Option<RawSentence>, GpsError<UART::Error>> {
        #[cfg(feature = "defmt")]
        debug!("GpsReader.read_sentence()");
        let mut buf = [0u8; SENTENCE_MAX_LEN];

        match uart.read(&mut buf).await.map_err(GpsError::Uart) {
            Ok(len) => {
                let mut res = Ok(None);
                if len > 0 {
                    buf[..len].iter().for_each(|b| {
                        self.buffer[self.buffer_idx] = *b;

                        if *b == LINE_FEED {
                            #[cfg(feature = "defmt")]
                            debug!("raw sentence: {}", &self.buffer[..=self.buffer_idx]);
                            match <Vec<u8, SENTENCE_MAX_LEN>>::try_from(&self.buffer[..=self.buffer_idx]) {
                                Ok(v) => {
                                    res = match RawSentence::from_utf8(v) {
                                        Ok(raw) => Ok(Some(raw)),
                                        Err(_) => Err(GpsError::Utf8)
                                    };
                                    self.reset_buffer();
                                }
                                Err(_) => res = Err(GpsError::Unexpected)
                            }
                        } else {
                            if self.buffer_idx + 1 == SENTENCE_MAX_LEN {
                                self.reset_buffer();
                            } else {
                                self.buffer_idx += 1;
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
        #[cfg(feature = "defmt")]
        debug!("GpsReader.reset_buffer()");
        self.buffer = [0; SENTENCE_MAX_LEN];
        self.buffer_idx = 0;
    }
}