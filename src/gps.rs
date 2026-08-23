use embedded_io_async::{ErrorType, Read};
use heapless::Vec;
use nmea::Nmea;
use pmtk::response::PmtkResponse;
use crate::{RawSentence, LINE_FEED, SENTENCE_MAX_LEN, GpsResponse};
use crate::error::GpsError;

pub(crate) struct GpsReader {
    buffer: [u8; SENTENCE_MAX_LEN],
    buffer_idx: usize,
}

impl Default for GpsReader {
    fn default() -> Self {
        Self { buffer: [0u8; SENTENCE_MAX_LEN], buffer_idx: 0 }
    }
}

impl GpsReader {
    pub async fn parse_sentence<UART: Read + ErrorType>(&mut self, nmea: &mut Nmea, sentence: &RawSentence) -> Result<GpsResponse, GpsError<UART::Error>> {
        match self.parse_nmea_sentence::<UART>(nmea, sentence).await {
            Ok(res) => Ok(res),
            Err(_) => self.parse_pmtk_sentence::<UART>(sentence).await,
        }
    }

    async fn parse_nmea_sentence<UART: Read + ErrorType>(&mut self, nmea: &mut Nmea, sentence: &RawSentence) -> Result<GpsResponse, GpsError<UART::Error>> {
        Ok(GpsResponse::Nmea(nmea.parse(sentence).map_err(|_| GpsError::Nmea)?))
    }

    async fn parse_pmtk_sentence<UART: Read + ErrorType>(&mut self, sentence: &RawSentence) -> Result<GpsResponse, GpsError<UART::Error>> {
        Ok(GpsResponse::Pmtk(PmtkResponse::try_from(sentence.as_bytes())?))
    }

    pub async fn read_sentence<UART: Read + ErrorType>(&mut self, uart: &mut UART) -> Result<Option<RawSentence>, GpsError<UART::Error>> {
        let mut buf = [0u8; SENTENCE_MAX_LEN];

        match uart.read(&mut buf).await.map_err(GpsError::Uart) {
            Ok(len) => {
                let mut res = Ok(None);
                if len > 0 {
                    buf[..len].iter().for_each(|b| {
                        self.buffer[self.buffer_idx] = *b;

                        if *b == LINE_FEED {
                            let v = <Vec<u8, SENTENCE_MAX_LEN>>::try_from(&self.buffer[..=self.buffer_idx]).unwrap(); // TODO remove unwrap()
                            res = Ok(Some(RawSentence::from_utf8(v).unwrap())); // TODO remove unwrap()
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
        self.buffer = [0; 255];
        self.buffer_idx = 0;
    }
}