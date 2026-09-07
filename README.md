# Adafruit Ultimate GPS
`#![no-std]`, `async`-first driver and toolkit for the [Adafruit Ultimate GPS breakout](https://www.adafruit.com/product/746).

### Features

- `defmt`: configure desired host log level with `$ export DEFMT_LOG=info`

### Examples

- [RP235x half-duplex](https://github.com/ardentTech/adafruit-ultimate-gps/blob/main/examples/rp235x/async/src/bin/half_duplex.rs)
- [RP235x full-duplex](https://github.com/ardentTech/adafruit-ultimate-gps/blob/main/examples/rp235x/async/src/bin/full_duplex.rs)

### TODO

- [x] `defmt` feature
- [ ] LOCUS integration
- [ ] `driver.rs` verify flag for requests
- [x] refactor `unwrap()`s
- [x] `full_duplex` example
- [ ] knock out TODOs


### License

* [MIT](https://github.com/ardentTech/adafruit-ultimate-gps/blob/main/LICENSE-MIT)
* [Apache](https://github.com/ardentTech/adafruit-ultimate-gps/blob/main/LICENSE-APACHE)