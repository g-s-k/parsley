# parsley

[![Build Status](https://travis-ci.org/g-s-k/parsley.svg?branch=master)](https://travis-ci.org/g-s-k/parsley)

what if scheme...but rust. still working on a backronym.

`cargo install parsley` installs the interpreter - see the
[docs](https://docs.rs/parsley) if you want to use it as a library.

## on the horizon

- improve test coverage, especially e2e tests
- (eventually) build on stable. this is blocked by the `box_patterns` feature,
  which is nice to use, but possible to work around - and it looks unlikely that
  it will land in stable any time soon.
- (very eventually) R7RS compliance

## License

Licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](./LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](./LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
