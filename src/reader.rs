use crate::error::GpsError;
use crate::types::SENTENCE_MAX_LEN;
use crate::types::{GpsResponse, RawSentence};
#[cfg(feature = "defmt")]
use defmt::debug;
use embedded_io_async::{ErrorType, Read};
use heapless::{String, Vec};
use nmea::Nmea;
use pmtk::response::PmtkResponse;

const LINE_FEED: u8 = 0x0a; // '\n'

pub(crate) struct GpsReader {
    buffer: [u8; SENTENCE_MAX_LEN],
    buffer_idx: usize,
    nmea: Nmea,
}

impl Default for GpsReader {
    fn default() -> Self {
        Self { buffer: [0u8; SENTENCE_MAX_LEN], buffer_idx: 0, nmea: Nmea::default() }
    }
}

impl GpsReader {
    pub(crate) async fn parse_sentence<UART: Read + ErrorType>(&mut self, sentence: &RawSentence) -> Result<GpsResponse, GpsError<UART::Error>> {
        #[cfg(feature = "defmt")]
        debug!("GpsReader.parse_sentence()");
        match self.parse_nmea_sentence::<UART>(sentence).await {
            Ok(res) => Ok(res),
            Err(_) => self.parse_pmtk_sentence::<UART>(sentence).await,
        }
    }

    async fn parse_nmea_sentence<UART: Read + ErrorType>(&mut self, sentence: &RawSentence) -> Result<GpsResponse, GpsError<UART::Error>> {
        #[cfg(feature = "defmt")]
        debug!("GpsReader.parse_nmea_sentence()");
        Ok(GpsResponse::Nmea(self.nmea.parse(sentence).map_err(|_| GpsError::Nmea)?))
    }

    async fn parse_pmtk_sentence<UART: Read + ErrorType>(&mut self, sentence: &RawSentence) -> Result<GpsResponse, GpsError<UART::Error>> {
        #[cfg(feature = "defmt")]
        debug!("GpsReader.parse_pmtk_sentence()");
        Ok(GpsResponse::Pmtk(PmtkResponse::try_from(sentence.as_bytes())?))
    }

    pub(crate) async fn read_response<UART: Read + ErrorType>(&mut self, uart: &mut UART) -> Result<Option<GpsResponse>, GpsError<UART::Error>> {
        #[cfg(feature = "defmt")]
        debug!("GpsReader.read_response()");
        match self.read_sentence(uart).await {
            Ok(res) => if let Some(raw) = res {
                match self.parse_sentence::<UART>(&raw).await {
                    Ok(res) => Ok(Some(res)),
                    Err(e) => Err(e)
                }
            } else { Ok(None) },
            Err(e) => Err(e)
        }
    }

    pub(crate) async fn read_sentence<UART: Read + ErrorType>(&mut self, uart: &mut UART) -> Result<Option<RawSentence>, GpsError<UART::Error>> {
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
                            let v = <Vec<u8, SENTENCE_MAX_LEN>>::try_from(&self.buffer[..=self.buffer_idx]).unwrap(); // TODO remove unwrap()
                            res = match RawSentence::from_utf8(v) {
                                Ok(raw) => Ok(Some(raw)),
                                Err(e) => Err(GpsError::Utf8(e))
                            };
                            self.reset_buffer();
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
        self.buffer = [0; 255];
        self.buffer_idx = 0;
    }
}