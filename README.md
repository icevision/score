# icevision-score [![Build Status](https://travis-ci.org/icevision/score.svg?branch=master)](https://travis-ci.org/icevision/score) [![dependency status](https://deps.rs/repo/github/icevision/score/status.svg)](https://deps.rs/repo/github/icevision/score)

Scoring software used in the IceVision competition.

## Usage example
```sh
$ cargo build --release
$ ./target/release/icevision-score file_examples/solution1.tsv file_examples/good.tsv
Total score:    2.447
Penalty:        0.000
Score 5.19:     1.666
Score 3.1:      0.781
Penalty 5.19:   0.000
Penalty 3.1:    0.000
```

## License

Licensed under either of

 * [Apache License, Version 2.0](http://www.apache.org/licenses/LICENSE-2.0)
 * [MIT license](http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
