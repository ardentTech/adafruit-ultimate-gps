# Adafruit Ultimate GPS
`#![no-std]`, `async`-first driver and toolkit for the [Adafruit Ultimate GPS breakout](https://www.adafruit.com/product/746).

### Usage

##### Half-Duplex

A simple half-duplex implementation is provided via the `AdafruitUltimateGps` driver. It combines PMTK GGA, RMC and GLL
sentences and supports sending PMTK commands. See [here](https://github.com/ardentTech/adafruit-ultimate-gps/blob/main/examples/rp235x/async/src/bin/half_duplex.rs) for a full example.

```rust
let mut gps = AdafruitUltimateGps::new(uart);
gps.start(1_000).await.unwrap(); // set NMEA update rate ms
match gps.read().await {
    Ok(res) => {},
    Err(e) => {},
}
```

##### Custom

If you need full-duplex, want to handle PMTK sentences beyond GGA, RMC and GLL or have other requirements, it's easy to
roll your own driver using the `GpsReader` and `GpsWriter` structs. See [here](https://github.com/ardentTech/adafruit-ultimate-gps/blob/main/examples/rp235x/async/src/bin/full_duplex.rs) for an example of a full-duplex
driver, or refer to the built-in [half-duplex driver](https://github.com/ardentTech/adafruit-ultimate-gps/blob/main/src/drivers.rs) as a starting point.

### Features

- `defmt`: configure desired host log level with `$ export DEFMT_LOG=info`

### Examples

- [RP235x async half-duplex](https://github.com/ardentTech/adafruit-ultimate-gps/blob/main/examples/rp235x/async/src/bin/half_duplex.rs)
- [RP235x async full-duplex](https://github.com/ardentTech/adafruit-ultimate-gps/blob/main/examples/rp235x/async/src/bin/full_duplex.rs)

### TODO

- [x] `defmt` feature
- [x] `driver.rs` verify flag for requests
- [x] refactor `unwrap()`s
- [x] `full_duplex` example
- [ ] knock out TODOs
- [ ] `sync` feature
- [ ] sync examples


### License

* [MIT](https://github.com/ardentTech/adafruit-ultimate-gps/blob/main/LICENSE-MIT)
* [Apache](https://github.com/ardentTech/adafruit-ultimate-gps/blob/main/LICENSE-APACHE)