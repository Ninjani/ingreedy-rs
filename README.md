# ingreedy-rs

[![Crates.io](https://img.shields.io/crates/v/ingreedy-rs.svg)](https://crates.io/crates/ingreedy-rs)
[![Docs.rs](https://docs.rs/ingreedy-rs/badge.svg)](https://docs.rs/ingreedy-rs)
[![CI](https://github.com/Ninjani/ingreedy-rs/workflows/Continuous%20Integration/badge.svg)](https://github.com/Ninjani/ingreedy-rs/actions)
[![Coverage Status](https://coveralls.io/repos/github/Ninjani/ingreedy-rs/badge.svg?branch=main)](https://coveralls.io/github/Ninjani/ingreedy-rs?branch=main)

## Natural language parsing of recipe ingredients

Rust port of [ingreedy-py](https://github.com/openculinary/ingreedy-py) which is a port of [Ingreedy](https://github.com/iancanderson/ingreedy-js). 


"2 (28 ounce) can crushed tomatoes"

```json
{
  "quantities": [
    {
      "amount": 56.0,
      "unit": "ounce",
      "unit_type": "English"
    }
  ],
  "ingredient": "can crushed tomatoes"
}
```

## As a Rust library

```toml
[dependencies]
ingreedy-rs = {version = "0.1.0", default-features = false}
```

```rust
use ingreedy_rs::Ingredient;

fn main() {
    let ingredient = Ingredient::parse("2 (28 ounce) can crushed tomatoes")?;
}
```

## As a command-line tool
Grab binaries from [releases](https://github.com/Ninjani/ingreedy-rs/releases/latest) or `cargo install ingreedy-rs`

```shell
ingreedy-rs "2 (28 ounce) can crushed tomatoes"
```

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

See [CONTRIBUTING.md](CONTRIBUTING.md).
