#![no_std]

mod error;
mod uart;
pub mod half_duplex;
pub mod full_duplex;
mod gps;

use defmt::Format;
use heapless::String;
use nmea::SentenceType;

pub use pmtk;
use pmtk::response::PmtkResponse;

const LINE_FEED: u8 = 0x0a; // '\n'
const SENTENCE_MAX_LEN: usize = 255;

pub type RawSentence = String<SENTENCE_MAX_LEN>;

#[derive(Debug, Format)]
pub enum GpsResponse {
    Nmea(SentenceType),
    Pmtk(PmtkResponse)
}