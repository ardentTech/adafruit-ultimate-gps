//! This example demonstrates how to implement a full-duplex AdafruitUltimateGps driver using
//! `GpsReader` and `GpsWriter` on a [RP Pico 2W](https://www.adafruit.com/product/6087).

#![no_std]
#![no_main]

use cortex_m::delay::Delay;
use defmt::{error, info};
use defmt_rtt as _;
use embedded_io::{ErrorType, Read, Write};
// Ensure we halt the program on panic (if we don't mention this crate it won't
// be linked)
use panic_halt as _;

// Alias for our HAL crate
use rp235x_hal as hal;

// Some things we need
use rp235x_hal::Clock;
use rp235x_hal::fugit::RateExtU32;
use rp235x_hal::multicore::{Multicore, Stack};
use rp235x_hal::uart::{DataBits, StopBits, UartConfig};
use adafruit_ultimate_gps::driver::AdafruitUltimateGps;
use adafruit_ultimate_gps::pmtk;
use adafruit_ultimate_gps::pmtk::dt::nmea_output::Frequency;
use adafruit_ultimate_gps::pmtk::q::release::ReleaseQ;
use adafruit_ultimate_gps::pmtk::traits::CmdQ;
use adafruit_ultimate_gps::reader::GpsReader;
use adafruit_ultimate_gps::types::{GpsError, GpsResponse};
use adafruit_ultimate_gps::writer::GpsWriter;

/// Tell the Boot ROM about our application
#[unsafe(link_section = ".start_block")]
#[used]
pub static IMAGE_DEF: hal::block::ImageDef = hal::block::ImageDef::secure_exe();

/// Stack for core 1
///
/// Core 0 gets its stack via the normal route - any memory not used by static
/// values is reserved for stack and initialised by cortex-m-rt.
/// To get the same for Core 1, we would need to compile everything seperately
/// and modify the linker file for both programs, and that's quite annoying.
/// So instead, core1.spawn takes a [usize] which gets used for the stack.
/// NOTE: We use the `Stack` struct here to ensure that it has 32-byte
/// alignment, which allows the stack guard to take up the least amount of
/// usable RAM.
static CORE1_STACK: Stack<4096> = Stack::new();

/// External high-speed crystal on the Raspberry Pi Pico 2 board is 12 MHz.
/// Adjust if your board has a different frequency
const XTAL_FREQ_HZ: u32 = 12_000_000u32;

// handles UART reads
struct GpsRx<UART> {
    rx: GpsReader,
    uart: UART,
}
impl<UART: Read + ErrorType> GpsRx<UART> {
    fn new(uart: UART) -> Self {
        Self { rx: GpsReader::default(), uart }
    }

    fn read(&mut self) -> Result<Option<GpsResponse>, GpsError<UART::Error>> {
        self.rx.read_response(&mut self.uart)
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

    pub fn send(&mut self, command: impl CmdQ) -> Result<(), GpsError<UART::Error>> {
        self.tx.send(&mut self.uart, command)
    }
}

/// Entry point to our bare-metal application.
///
/// The `#[hal::entry]` macro ensures the Cortex-M start-up code calls this function
/// as soon as all global variables and the spinlock are initialised.
///
/// The function configures the rp235x peripherals, then toggles a GPIO pin in
/// an infinite loop. If there is an LED connected to that pin, it will blink.
#[hal::entry]
fn main() -> ! {
    // Grab our singleton objects
    let mut pac = hal::pac::Peripherals::take().unwrap();

    // Set up the watchdog driver - needed by the clock setup code
    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);

    // Configure the clocks
    let clocks = hal::clocks::init_clocks_and_plls(
        XTAL_FREQ_HZ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
        .unwrap();

    // The single-cycle I/O block controls our GPIO pins
    let mut sio = hal::Sio::new(pac.SIO);

    // Set the pins to their default state
    let pins = hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let uart0_pins = (
        pins.gpio16.into_function(),
        pins.gpio17.into_function(),
    );
    let uart = hal::uart::UartPeripheral::new(pac.UART0, uart0_pins, &mut pac.RESETS)
        .enable(
            UartConfig::new(9600.Hz(), DataBits::Eight, None, StopBits::One),
            clocks.peripheral_clock.freq(),
        )
        .unwrap();
    let (rx, tx) = uart.split();
    let mut gps_rx = GpsRx::new(rx);
    let mut gps_tx = GpsTx::new(tx);

    // Start up the second core to tx some queries after a delay
    let mut mc = Multicore::new(&mut pac.PSM, &mut pac.PPB, &mut sio.fifo);
    let cores = mc.cores();
    let core1 = &mut cores[1];
    core1
        .spawn(CORE1_STACK.take().unwrap(), move || {
            // Get the second core's copy of the `CorePeripherals`, which are per-core.
            // Unfortunately, `cortex-m` doesn't support this properly right now,
            // so we have to use `steal`.
            let core = unsafe { cortex_m::Peripherals::steal() };
            // Set up the delay for the second core.
            let sys_freq = clocks.system_clock.freq().to_Hz();
            let mut delay = Delay::new(core.SYST, sys_freq);

            delay.delay_ms(1_000);
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
            ).ok();

            gps_tx.send(
                pmtk::cmd::set_nmea_update_rate::SetNmeaUpdateRateCmd::new(1_000).unwrap()
            ).ok();

            delay.delay_ms(3_000);

            gps_tx.send(ReleaseQ {}).ok();
        })
        .unwrap();

    loop {
        // read parsed sentences
        match gps_rx.read() {
            Ok(response) => info!("gps.read ok: {:?}", response),
            Err(e) => error!("gps.read err: {:?}", e),
        }
    }
}

/// Program metadata for `picotool info`
#[unsafe(link_section = ".bi_entries")]
#[used]
pub static PICOTOOL_ENTRIES: [hal::binary_info::EntryAddr; 5] = [
    hal::binary_info::rp_cargo_bin_name!(),
    hal::binary_info::rp_cargo_version!(),
    hal::binary_info::rp_program_description!(c"Adafruit Ultimate GPS Full-Duplex Sync Example"),
    hal::binary_info::rp_cargo_homepage_url!(),
    hal::binary_info::rp_program_build_attribute!(),
];

// End of file
