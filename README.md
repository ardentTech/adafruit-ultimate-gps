# Adafruit Ultimate GPS
`#![no-std]`, `async`-first driver and toolkit for the [Adafruit Ultimate GPS breakout](https://www.adafruit.com/product/746).

### Features

- `defmt`: configure desired host log level with `$ export DEFMT_LOG=info`

### TODO

- [x] `defmt` feature
- [ ] `full_duplex` feature (should be default?)
- [ ] LOCUS integration
- [ ] verify flag for requests
- [ ] refactor `unwrap()`s