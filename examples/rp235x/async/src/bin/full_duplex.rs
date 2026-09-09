//! This example demonstrates how to implement a full-duplex AdafruitUltimateGps driver using
//! `GpsReader` and `GpsWriter` on a [RP Pico 2W](https://www.adafruit.com/product/6087).

#![no_std]
#![no_main]

use defmt::{error, info};
#[allow(unused_imports)]
use {defmt_rtt as _, panic_probe as _};
use adafruit_ultimate_gps::pmtk;
use adafruit_ultimate_gps::pmtk::dt::nmea_output::Frequency;
use adafruit_ultimate_gps::pmtk::q::release::ReleaseQ;
use adafruit_ultimate_gps::pmtk::traits::CmdQ;
use adafruit_ultimate_gps::reader::GpsReader;
use adafruit_ultimate_gps::types::GpsError;
use adafruit_ultimate_gps::types::{GpsResponse, RawSentence};
use adafruit_ultimate_gps::writer::GpsWriter;
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::UART0;
use embassy_rp::uart::{BufferedInterruptHandler, BufferedUart, BufferedUartRx, BufferedUartTx, Config};
use embassy_time::Timer;
use embedded_io_async::{ErrorType, Read, Write};
use static_cell::StaticCell;

// handles UART reads
struct GpsRx<UART> {
    rx: GpsReader,
    uart: UART,
}
impl<UART: Read + ErrorType> GpsRx<UART> {
    fn new(uart: UART) -> Self {
        Self { rx: GpsReader::default(), uart }
    }

    async fn read(&mut self) -> Result<Option<GpsResponse>, GpsError<UART::Error>> {
        self.rx.read_response(&mut self.uart).await
    }
}

// handles UART writes
struct GpsTx<UART> {
    tx: GpsWriter,
    uart: UART,
}

impl<UART: Write + ErrorType> GpsTx<UART> {
    pub fn new(uart: UART) -> Self {
        Self { uart, tx: GpsWriter {} }
    }

    pub async fn send(&mut self, command: impl CmdQ) -> Result<(), GpsError<UART::Error>> {
        self.tx.send(&mut self.uart, command).await
    }
}

bind_interrupts!(struct Irqs {
    UART0_IRQ => BufferedInterruptHandler<UART0>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let (tx_pin, rx_pin, uart) = (p.PIN_16, p.PIN_17, p.UART0);

    static TX_BUF: StaticCell<[u8; 16]> = StaticCell::new();
    let tx_buf = &mut TX_BUF.init([0; 16])[..];
    static RX_BUF: StaticCell<[u8; 16]> = StaticCell::new();
    let rx_buf = &mut RX_BUF.init([0; 16])[..];
    let mut config = Config::default();
    config.baudrate = 9600;
    let uart = BufferedUart::new(uart, tx_pin, rx_pin, Irqs, tx_buf, rx_buf, config);

    let (tx, rx) = uart.split();
    let gps_rx = GpsRx::new(rx);
    let gps_tx = GpsTx::new(tx);

    spawner.spawn(gps_rx_task(gps_rx).unwrap());
    spawner.spawn(gps_tx_task(gps_tx).unwrap());
}

#[embassy_executor::task]
async fn gps_rx_task(mut gps_rx: GpsRx<BufferedUartRx>) {
    info!("gps_rx_task");
    loop {
        // read parsed sentences
        match gps_rx.read().await {
            Ok(Some(res)) => info!("{:?}", res),
            Ok(None) => {}
            Err(e) => error!("{:?}", e),
        }
    }
}

#[embassy_executor::task]
async fn gps_tx_task(mut gps_tx: GpsTx<BufferedUartTx>) {
    info!("gps_tx_task");

    gps_tx.send(
        pmtk::cmd::set_nmea_output::SetNmeaOutputCmd::new(
            Frequency::Disabled,
            Frequency::OnceEveryFivePositionFixes,
            Frequency::Disabled,
            Frequency::OnceEveryFivePositionFixes,
            Frequency::Disabled,
            Frequency::Disabled,
            Frequency::Disabled,
        )
    ).await.ok();

    gps_tx.send(
        pmtk::cmd::set_nmea_update_rate::SetNmeaUpdateRateCmd::new(1_000).unwrap()
    ).await.ok();

    Timer::after_millis(3_000).await;
    gps_tx.send(ReleaseQ {}).await.ok();
}