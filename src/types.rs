use heapless::String;
use nmea::SentenceType;
use pmtk::response::PmtkResponse;

pub type RawSentence = String<SENTENCE_MAX_LEN>;

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, PartialEq)]
pub enum GpsResponse {
    Nmea(SentenceType),
    Pmtk(PmtkResponse)
}

pub const SENTENCE_MAX_LEN: usize = 255;