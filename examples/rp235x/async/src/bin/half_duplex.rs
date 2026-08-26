#![no_std]
#![no_main]

use defmt::{error, info, warn};
#[allow(unused_imports)]
use {defmt_rtt as _, panic_probe as _};
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::UART0;
use embassy_rp::uart::{BufferedInterruptHandler, BufferedUart, Config};
use static_cell::StaticCell;
use adafruit_ultimate_gps::{pmtk as pmtk};
use pmtk::dt::nmea_output::Frequency;
use adafruit_ultimate_gps::half_duplex::Gps;

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

    let mut gps = Gps::new(uart);

    let cmd = pmtk::cmd::set_nmea_output::SetNmeaOutputCmd::new(
        Frequency::OnceEveryFivePositionFixes,
        Frequency::Disabled,
        Frequency::Disabled,
        Frequency::Disabled,
        Frequency::Disabled,
        Frequency::Disabled,
        Frequency::Disabled,
    );
    // send command and verify
    match gps.send_and_verify(cmd, 8).await {
        Ok(v) => if v { info!("verified: {}", cmd) } else { warn!("unverified: {}", cmd) },
        Err(e) => error!("{:?}", e),
    }

    // send command and do not verify
    let cmd = pmtk::cmd::set_nmea_update_rate::SetNmeaUpdateRateCmd::new(1_000).unwrap();
    gps.send(cmd).await.unwrap();

    loop {
        // read parsed sentences
        match gps.read().await {
            Ok(Some(response)) => info!("gps.read ok: {:?}", response),
            Ok(None) => info!("gps.read none"),
            Err(e) => error!("gps.read err: {:?}", e),
        }
    }
}