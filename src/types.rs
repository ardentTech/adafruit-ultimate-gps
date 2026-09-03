#[cfg(feature = "defmt")]
use defmt::Format;
use heapless::String;
use nmea::{ParseResult, SentenceType};
use pmtk::response::PmtkResponse;

pub type RawSentence = String<SENTENCE_MAX_LEN>;

#[derive(Debug, PartialEq)]
pub enum GpsResponse {
    Nmea(ParseResult),
    Pmtk(PmtkResponse)
}

#[cfg(feature = "defmt")]
impl Format for GpsResponse {
    fn format(&self, fmt: defmt::Formatter) {
        match self {
            GpsResponse::Nmea(sentence) => match sentence {
                ParseResult::GGA(data) => defmt::write!(fmt, "NMEA {}", data),
                ParseResult::RMC(data) => defmt::write!(fmt, "NMEA {}", data),
                _ => defmt::write!(fmt, "NMEA {:?}", sentence),
            }
            GpsResponse::Pmtk(response) => match response {
                PmtkResponse::Ack(dt) => defmt::write!(fmt, "PMTK {:?}", dt),
                PmtkResponse::SysMsg(dt) => defmt::write!(fmt, "PMTK {:?}", dt),
                PmtkResponse::TxtMsg(dt) => defmt::write!(fmt, "PMTK {:?}", dt),
                PmtkResponse::DgpsMode(dt) => defmt::write!(fmt, "PMTK {:?}", dt),
                PmtkResponse::SbasEnabled(dt) => defmt::write!(fmt, "PMTK {:?}", dt),
                PmtkResponse::NmeaOutput(dt) => defmt::write!(fmt, "PMTK {:?}", dt),
                PmtkResponse::SbasMode(dt) => defmt::write!(fmt, "PMTK {:?}", dt),
                PmtkResponse::NavThreshold(dt) => defmt::write!(fmt, "PMTK {:?}", dt),
                PmtkResponse::Release(dt) => defmt::write!(fmt, "PMTK {:?}", dt),
                PmtkResponse::EpoInfo(dt) => defmt::write!(fmt, "PMTK {:?}", dt),
                PmtkResponse::EasyEnable(dt) => defmt::write!(fmt, "PMTK {:?}", dt),
                PmtkResponse::Log(dt) => defmt::write!(fmt, "PMTK {:?}", dt),
                PmtkResponse::Lox(dt) => defmt::write!(fmt, "PMTK {:?}", dt),
            }
        }
    }
}

pub const SENTENCE_MAX_LEN: usize = 255;