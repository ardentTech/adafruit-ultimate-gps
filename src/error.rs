use core::str::Utf8Error;
use defmt::Format;
use pmtk::error::PmtkError;

#[derive(Debug, Format)]
pub enum GpsError<UART> {
    Nmea, // TODO wrap
    Overflow,
    Pmtk, // TODO wrap
    PmtkParseDt,
    Uart(UART),
    Utf8, // TODO needs to be stand-alone?
    UnexpectedNumBytes,
}

impl<UART> From<Utf8Error> for GpsError<UART> {
    fn from(_: Utf8Error) -> Self {
        Self::Utf8
    }
}

impl<UART> From<PmtkError> for GpsError<UART> {
    fn from(_: PmtkError) -> Self {
        Self::Pmtk
    }
}