#![no_std]
#![no_main]

use defmt::{error, info};
#[allow(unused_imports)]
use {defmt_rtt as _, panic_probe as _};
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::UART0;
use embassy_rp::uart::{BufferedInterruptHandler, BufferedUart, BufferedUartRx, BufferedUartTx, Config};
use embassy_time::Timer;
use static_cell::StaticCell;
use adafruit_ultimate_gps::full_duplex::{GpsRx, GpsTx};
use adafruit_ultimate_gps::pmtk;
use adafruit_ultimate_gps::pmtk::cmd::full_cold_start::FullColdStartCmd;
use adafruit_ultimate_gps::pmtk::dt::nmea_output::Frequency;

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
    info!("gps_rx task");
    loop {
        // read raw sentences
        match gps_rx.read_raw().await {
            Ok(Some(raw)) => info!("{:?}", raw),
            Ok(None) => {}
            Err(e) => error!("{:?}", e),
        }
    }
}

#[embassy_executor::task]
async fn gps_tx_task(mut gps_tx: GpsTx<BufferedUartTx>) {
    info!("gps_tx task");

    // "It’s essentially a Cold Restart, but additionally clear system/user configurations at
    // re-start. That is, reset the receiver to the factory status."
    gps_tx.send(FullColdStartCmd {}).await.ok();

    gps_tx.send(
        pmtk::cmd::set_nmea_output::SetNmeaOutputCmd::new(
            Frequency::OnceEveryFivePositionFixes,
            Frequency::Disabled,
            Frequency::Disabled,
            Frequency::Disabled,
            Frequency::Disabled,
            Frequency::Disabled,
            Frequency::Disabled,
        )
    ).await.ok();

    gps_tx.send(
        pmtk::cmd::set_nmea_update_rate::SetNmeaUpdateRateCmd::new(1_000).unwrap()
    ).await.ok();

    gps_tx.erase_logger().await.ok();
    gps_tx.set_logger_interval(15).await.ok();
    gps_tx.start_logger().await.ok();

    Timer::after_secs(5).await;
    //gps_tx.logger_status().await.ok();
    gps_tx.query_logger().await.ok();
}