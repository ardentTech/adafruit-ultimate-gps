use embedded_io_async::{ErrorType, Read, Write};
use heapless::Vec;
use nmea::Nmea;
use pmtk::response::PmtkResponse;
use pmtk::traits::{Cmd, Q};
use crate::{RawSentence, LINE_FEED, SENTENCE_MAX_LEN, GpsResponse, uart};
use crate::error::GpsError;

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
        match self.parse_nmea_sentence::<UART>(sentence).await {
            Ok(res) => Ok(res),
            Err(_) => self.parse_pmtk_sentence::<UART>(sentence).await,
        }
    }

    async fn parse_nmea_sentence<UART: Read + ErrorType>(&mut self, sentence: &RawSentence) -> Result<GpsResponse, GpsError<UART::Error>> {
        Ok(GpsResponse::Nmea(self.nmea.parse(sentence).map_err(|_| GpsError::Nmea)?))
    }

    async fn parse_pmtk_sentence<UART: Read + ErrorType>(&mut self, sentence: &RawSentence) -> Result<GpsResponse, GpsError<UART::Error>> {
        Ok(GpsResponse::Pmtk(PmtkResponse::try_from(sentence.as_bytes())?))
    }

    pub(crate) async fn read_response<UART: Read + ErrorType>(&mut self, uart: &mut UART) -> Result<Option<GpsResponse>, GpsError<UART::Error>> {
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

pub(crate) struct GpsWriter {}

impl GpsWriter {
    /// Sends a PMTK command.
    pub(crate) async fn send_command<UART: Write + ErrorType>(&mut self, uart: &mut UART, command: impl Cmd) -> Result<(), GpsError<UART::Error>> {
        uart::write(uart, command.serialize()?.as_bytes()).await
    }

    /// Sends a PMTK query.
    pub(crate) async fn send_query<UART: Write + ErrorType>(&mut self, uart: &mut UART, query: impl Q) -> Result<(), GpsError<UART::Error>> {
        uart::write(uart, query.serialize()?.as_bytes()).await
    }
}