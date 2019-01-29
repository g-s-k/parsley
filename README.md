# parsley

[![Build Status](https://travis-ci.org/g-s-k/parsley.svg?branch=master)](https://travis-ci.org/g-s-k/parsley)
[![Crates.io](http://meritbadge.herokuapp.com/parsley)](https://crates.io/crates/parsley)

what if scheme...but rust. still working on a backronym.

`cargo install parsley` installs the interpreter - see the
[docs](https://docs.rs/parsley) if you want to use it as a library.

## high-level goals

1. a lightweight Scheme implementation...
2. that compiles to WebAssembly...
3. usable inside of a larger application without making too many sacrifices...
4. that is modular and extensible...
5. but includes enough definitions to be useful out of the box.

## on the horizon

- improve test coverage
  - More tests from SICP
  - Better coverage of the standard library
  - Example crates
- (eventually) R7RS compliance

## License

Licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](./LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](./LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
