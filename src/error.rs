use core::str::Utf8Error;
use pmtk::error::PmtkError;

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug)]
pub enum GpsError<UART> {
    Nmea, // TODO wrap nmea::Error (will need lifetime...)
    Pmtk(PmtkError),
    Uart(UART),
    Utf8(Utf8Error),
}

impl<UART> From<Utf8Error> for GpsError<UART> {
    fn from(e: Utf8Error) -> Self {
        Self::Utf8(e)
    }
}

impl<UART> From<PmtkError> for GpsError<UART> {
    fn from(e: PmtkError) -> Self {
        Self::Pmtk(e)
    }
}