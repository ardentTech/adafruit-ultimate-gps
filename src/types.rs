#[cfg(feature = "defmt")]
use defmt::Format;
use heapless::String;
use nmea::SentenceType;
use pmtk::response::PmtkResponse;

pub type RawSentence = String<SENTENCE_MAX_LEN>;

#[derive(Debug, PartialEq)]
pub enum GpsResponse {
    Nmea(SentenceType),
    Pmtk(PmtkResponse)
}

// pub latitude: Option<f64>,
// pub longitude: Option<f64>,
// #[cfg_attr(feature = "defmt", defmt(Debug2Format))]
// pub fix_time: Option<NaiveTime>,
// pub valid: bool,
// pub faa_mode: Option<FaaMode>,

#[cfg(feature = "defmt")]
impl Format for GpsResponse {
    fn format(&self, fmt: defmt::Formatter) {
        match self {
            GpsResponse::Nmea(sentence) => match sentence {
                SentenceType::GLL => defmt::write!(fmt, "NMEA GLL {} {}", sentence),
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