#![no_std]
#![no_main]

use defmt::{error, info};
#[allow(unused_imports)]
use {defmt_rtt as _, panic_probe as _};
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::UART0;
use embassy_rp::uart::{BufferedInterruptHandler, BufferedUart, BufferedUartRx, Config};
use embassy_time::Timer;
use static_cell::StaticCell;
use adafruit_ultimate_gps::{pmtk, GpsRx, GpsTx};

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
    let mut gps_tx = GpsTx::new(tx);

    spawner.spawn(gps_reader(gps_rx).unwrap());

    // Turn on the basic GGA and RMC info (what you typically want)
    let cmd = pmtk::cmd::set_nmea_output::SetNmeaOutputCmd::default();
    gps_tx.command(cmd).await.ok();

    // Set update rate to once a second (1hz) which is what you typically want.
    let cmd = pmtk::cmd::set_nmea_update_rate::SetNmeaUpdateRateCmd::new(1_000).unwrap();
    // TODO still not seeing Ack responses from either cmd above...
    gps_tx.command(cmd).await.ok();

    info!("entering main loop...");
    loop {
        gps_tx.command(cmd).await.ok();
        Timer::after_secs(3).await;
    }
}

#[embassy_executor::task]
async fn gps_reader(mut gps_rx: GpsRx<BufferedUartRx>) {
    info!("gps_reader task");
    loop {
        match gps_rx.read().await {
            Ok(Some(response)) => info!("gps.read ok: {:?}", response),
            Ok(None) => {}
            Err(e) => error!("gps.read err: {:?}", e),
        }
    }
}