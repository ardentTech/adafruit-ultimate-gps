#![no_std]
#![no_main]

use defmt::{error, info};
#[allow(unused_imports)]
use {defmt_rtt as _, panic_probe as _};
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::UART0;
use embassy_rp::uart::{BufferedInterruptHandler, BufferedUart, Config};
use static_cell::StaticCell;
use adafruit_ultimate_gps::driver::AdafruitGPS;

bind_interrupts!(struct Irqs {
    UART0_IRQ => BufferedInterruptHandler<UART0>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let (tx_pin, rx_pin, uart) = (p.PIN_16, p.PIN_17, p.UART0);

    static TX_BUF: StaticCell<[u8; 16]> = StaticCell::new();
    let tx_buf = &mut TX_BUF.init([0; 16])[..];
    static RX_BUF: StaticCell<[u8; 16]> = StaticCell::new();
    let rx_buf = &mut RX_BUF.init([0; 16])[..];
    let mut config = Config::default();
    config.baudrate = 9600;
    let uart = BufferedUart::new(uart, tx_pin, rx_pin, Irqs, tx_buf, rx_buf, config);

    let mut gps = AdafruitGPS::new(uart);
    gps.start(1_000).await.unwrap();

    loop {
        // read parsed sentences
        match gps.read().await {
            Ok(response) => info!("gps.read ok: {:?}", response),
            Err(e) => error!("gps.read err: {:?}", e),
        }
    }
}