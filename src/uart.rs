use embedded_io_async::{ErrorType, Read, Write};
use heapless::Vec;
use nmea::Nmea;
use pmtk::response::PmtkResponse;
use crate::error::GpsError;
use crate::error::GpsError::UnexpectedNumBytes;
use crate::{uart, GpsResponse, RawSentence, SENTENCE_MAX_LEN, LINE_FEED};

pub(crate) async fn parse_nmea_sentence<UART: Read + ErrorType>(nmea: &mut Nmea, sentence: &RawSentence) -> Result<GpsResponse, GpsError<UART::Error>> {
    Ok(GpsResponse::Nmea(nmea.parse(sentence).map_err(|_| GpsError::Nmea)?))
}

pub(crate) async fn parse_pmtk_sentence<UART: Read + ErrorType>(sentence: &RawSentence) -> Result<GpsResponse, GpsError<UART::Error>> {
    Ok(GpsResponse::Pmtk(PmtkResponse::try_from(sentence.as_bytes())?))
}

pub(crate) async fn parse_sentence<UART: Read + ErrorType>(nmea: &mut Nmea, sentence: &RawSentence) -> Result<GpsResponse, GpsError<UART::Error>> {
    match parse_nmea_sentence::<UART>(nmea, sentence).await {
        Ok(res) => Ok(res),
        Err(_) => parse_pmtk_sentence::<UART>(sentence).await,
    }
}

pub(crate) async fn read<UART: Read + ErrorType>(uart: &mut UART, buf: &mut [u8]) -> Result<usize, GpsError<UART::Error>> {
    uart.read(buf).await.map_err(GpsError::Uart)
}

pub(crate) async fn write<UART: Write + ErrorType>(uart: &mut UART, buf: &[u8]) -> Result<(), GpsError<UART::Error>> {
    if buf.len() != uart.write(buf).await.map_err(GpsError::Uart)? {
        return Err(UnexpectedNumBytes) // TODO not happy with the name of this error
    }
    Ok(())
}

pub(crate) async fn read_sentence<UART: Read + ErrorType>(
    uart: &mut UART,
    buffer: &mut [u8; SENTENCE_MAX_LEN],
    buffer_idx: &mut usize,
) -> Result<Option<RawSentence>, GpsError<UART::Error>> {
    let mut buf = [0u8; SENTENCE_MAX_LEN];

    match read(uart, &mut buf).await {
        Ok(len) => {
            let mut res = Ok(None);
            if len > 0 {
                buf[..len].iter().for_each(|b| {
                    buffer[*buffer_idx] = *b;

                    if *b == LINE_FEED {
                        let v = <Vec<u8, SENTENCE_MAX_LEN>>::try_from(&buffer[..=*buffer_idx]).unwrap(); // TODO remove unwrap()
                        res = Ok(Some(RawSentence::from_utf8(v).unwrap())); // TODO remove unwrap()
                        reset_read_buffer(buffer, buffer_idx);
                    } else {
                        if *buffer_idx + 1 == SENTENCE_MAX_LEN {
                            reset_read_buffer(buffer, buffer_idx);
                        } else {
                            *buffer_idx += 1;
                        }
                    }
                });
            }
            res
        }
        Err(e) => Err(e)
    }
}

fn reset_read_buffer(buffer: &mut [u8; SENTENCE_MAX_LEN], buffer_idx: &mut usize) {
    *buffer = [0u8; 255];
    *buffer_idx = 0;
}