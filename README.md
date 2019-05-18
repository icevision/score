# icevision-score [![Build Status](https://travis-ci.org/icevision/score.svg?branch=master)](https://travis-ci.org/icevision/score) [![dependency status](https://deps.rs/repo/github/icevision/score/status.svg)](https://deps.rs/repo/github/icevision/score)

Scoring software used in the IceVision competition.

## Usage example
We will use `training/` folder from [annotations] repository.
```sh
$ cargo build --release
$ ./icevision-score annotations/training/ file_examples/good.tsv
Total score:    1.783
Total penalty:  0.000
Score 2.1:  0.000
Score 2.4:  0.783
Score 3.1:  0.000
Score 3.24: 0.000
Score 3.27: 0.000
Score 4.1:  1.000
Score 4.2:  0.000
Score 5.19: 0.000
Score 5.20: 0.000
Score 8.22: 0.000
Penalty 2.1:    0.000
Penalty 2.4:    0.000
Penalty 3.1:    0.000
Penalty 3.24:   0.000
Penalty 3.27:   0.000
Penalty 4.1:    0.000
Penalty 4.2:    0.000
Penalty 5.19:   0.000
Penalty 5.20:   0.000
Penalty 8.22:   0.000
```

[annotations]: https://github.com/icevision/annotations/

## License

Licensed under either of

 * [Apache License, Version 2.0](http://www.apache.org/licenses/LICENSE-2.0)
 * [MIT license](http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
