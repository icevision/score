# icevision-score [![Build Status](https://travis-ci.org/icevision/score.svg?branch=master)](https://travis-ci.org/icevision/score) [![dependency status](https://deps.rs/repo/github/icevision/score/status.svg)](https://deps.rs/repo/github/icevision/score)

Scoring software used in the IceVision competition.

The version used during online stage had version [`v0.1.5`](https://github.com/icevision/score/tree/v0.1.5).

## Input file formats

For ground truth we use exactly the same format as in the [annotations]
repository.

For solutions we use a slightly different TSV (tab-separated values) file format. It contains all detected traffic signs of given classes on provided frame sequences. A solution file must contain header and the following fields:
- `frame`: sequence + frame number, e.g. `2018-02-13_1418_left/000032`.
- `xtl`, `ytl`, `xbr`, `ybr`: bounding box coordinates, integer or float. Note: `xtl` must be bigger than `xbr` and `ytl` must be bigger than `ybr`
- `class`: traffic sign code. Valid values are: `2.1`, `2.4`, `3.1`, `3.24`, `3.27`, `4.1`, `4.2`, `5.19`, `5.20`, `8.22`.
- `temporary`: is sign temporary (has a yellow background)? Valid values: `true` and `false`. Can be omitted.
- `data`: associated sign data. UTF-8 encoded string. Can be omitted.

Some examples can be found in [`examples/`] folder.

[`examples/`]: https://github.com/icevision/score/tree/master/src

## Scoring methodology

During offline stage participants can detect all traffic sign classes defined by Russian traffic code.

Bounding boxes with an area smaller than 100 pixels are ignored during
evaluation. Detection is considered successful if [IoU] is bigger or equal to
0.3 and bounding box has a correct class or superclass code. If sign is
detected twice, then detection with a smallest IoU will be counted as a false
positive. Each false positive or incorrect detection results in a penalty
equal to 2 points.

Score for true positives is calculated as `(1 + k1 + k2 + k3)*s`, where:
s -- "base" score, k1 -- coefficient for detecting sign code,
k2 -- coefficient for detecting associated data, k3 -- coefficient for
detecting temporary sign. If `(1 + k1 + k2 + k3) < 0`, detection score
is set to 0.

If IoU > 0.85, `s = 1`. Otherwise it is calculated using the following
equation: `((IoU – 0.3)/0.55)^0.25`.

`k1` is calculated as follows. For two-digit signs (e.g. “1.25”):
- If only one digit is detected (e.g. “1”), `k1=-0.7`
- If two digits are detected (e.g. “1.25”), `k1=0`

For three-digit signs (e.g. “5.19.1”):
- If only one digit is detected (e.g. “5”), `k1=-0.7`
- If two digits are detected (e.g. “5.19”), `k1=-0.2`
- If three digits are detected (e.g. “5.19.1”), `k1=0`

If manual annotation have used “8” for sign code, then for all traffic sign detections beginning with “8”, K1=0

If detection provides sign associated data and it is equal to annotation
associated data, `k2=2`. If data is different, `k2=-0.5`. If annotation has
used “NA” for associated data, `k2=0` for any data provided in detection.

If detection does not provide any information about sign being temporary
(empty string in the "temporary" field), `k3=0`. If sign is annotated as
temporary and detection is correct ("true" in the "temporary" field), `k3=1`.
If sign is not annotated as temporary and detection is correct ("false" in the
"temporary" field), `k3=0`. If detection is incorrect about sign being
temporary, `k3=-0.5`.

The final score is computed as sum of all true positive points minus all penalties.

For exact algorithm, please, refer to the code.

[IoU]: https://en.wikipedia.org/wiki/Jaccard_index

## Building
Scoring software is written in Rust, so you'll need to grab a
[Rust installation] in order to compile it. In general, we use the latest
stable release of the Rust compiler, but older versions may work as well.

```
$ git clone https://github.com/icevision/score
$ cd score/
$ cargo build --release
$ ./target/release/icevision-score --help
icevision-score 0.2.2
Artyom Pavlov <newpavlov@gmail.com>
IceVision competition scoring software

USAGE:
    icevision-score [FLAGS] <ground_truth> <solution>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information
    -v, --verbose    Enable verbose report

ARGS:
    <ground_truth>    Path to a directory with ground truth TSV files.
    <solution>        Path to a solution TSV file.
```


[Rust installation]: https://www.rust-lang.org/

## Usage example
For ground truth we use some files from [annotations] repository.
```
$ ./target/release/icevision-score examples/ground_truth/ examples/good.tsv
Total score:    1.249
Total penalty:  0.000
Per class results:
Class   Score   Penalty
4.1     0.791   0.000
2.4     0.458   0.000

```

To enable verbose report use `--verbose` flag:
```
$ ./target/release/icevision-score --verbose examples/ground_truth/ examples/good.tsv

frame: 2018-02-13_1418_left/000033
score   xtl     ytl     xbr     ybr     class   s       k1  k2  k3
0.791   1774    896     1847    979     4.1     0.989   -20 0   0
0.458   1643    895     1771    973     2.4     0.916   0   0   -50

===========================

Total score:    1.249
Total penalty:  0.000
Per class results:
Class   Score   Penalty
4.1     0.791   0.000
2.4     0.458   0.000

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

[2.1]: https://upload.wikimedia.org/wikipedia/commons/thumb/c/c4/2.1_Russian_road_sign.svg/100px-2.1_Russian_road_sign.svg.png
[2.4]: https://upload.wikimedia.org/wikipedia/commons/thumb/d/d1/2.4_Russian_road_sign.svg/100px-2.4_Russian_road_sign.svg.png
[3.1]: https://upload.wikimedia.org/wikipedia/commons/thumb/3/32/3.1_Russian_road_sign.svg/100px-3.1_Russian_road_sign.svg.png
[3.24]: https://upload.wikimedia.org/wikipedia/commons/thumb/d/d9/3.24_Russian_road_sign.svg/100px-3.24_Russian_road_sign.svg.png
[3.27]: https://upload.wikimedia.org/wikipedia/commons/thumb/9/98/3.27_Russian_road_sign.svg/100px-3.27_Russian_road_sign.svg.png
[4.1.1]: https://upload.wikimedia.org/wikipedia/commons/thumb/5/5b/4.1.1_Russian_road_sign.svg/100px-4.1.1_Russian_road_sign.svg.png
[4.1.2]: https://upload.wikimedia.org/wikipedia/commons/thumb/2/23/4.1.2_Russian_road_sign.svg/100px-4.1.2_Russian_road_sign.svg.png
[4.1.3]: https://upload.wikimedia.org/wikipedia/commons/thumb/4/46/4.1.3_Russian_road_sign.svg/100px-4.1.3_Russian_road_sign.svg.png
[4.1.4]: https://upload.wikimedia.org/wikipedia/commons/thumb/b/be/4.1.4_Russian_road_sign.svg/100px-4.1.4_Russian_road_sign.svg.png
[4.1.5]: https://upload.wikimedia.org/wikipedia/commons/thumb/7/73/4.1.5_Russian_road_sign.svg/100px-4.1.5_Russian_road_sign.svg.png
[4.1.6]: https://upload.wikimedia.org/wikipedia/commons/thumb/7/79/4.1.6_Russian_road_sign.svg/100px-4.1.6_Russian_road_sign.svg.png
[4.2.1]: https://upload.wikimedia.org/wikipedia/commons/thumb/c/c4/4.2.1_Russian_road_sign.svg/100px-4.2.1_Russian_road_sign.svg.png
[4.2.2]: https://upload.wikimedia.org/wikipedia/commons/thumb/9/96/4.2.2_Russian_road_sign.svg/100px-4.2.2_Russian_road_sign.svg.png
[4.2.3]: https://upload.wikimedia.org/wikipedia/commons/thumb/7/72/4.2.3_Russian_road_sign.svg/100px-4.2.3_Russian_road_sign.svg.png
[5.19.1]: https://upload.wikimedia.org/wikipedia/commons/thumb/b/b5/5.19.1_Russian_road_sign.svg/100px-5.19.1_Russian_road_sign.svg.png
[5.19.2]: https://upload.wikimedia.org/wikipedia/commons/thumb/0/07/5.19.2_Russian_road_sign.svg/100px-5.19.2_Russian_road_sign.svg.png
[5.20]: https://upload.wikimedia.org/wikipedia/commons/thumb/4/4e/5.20_Russian_road_sign.svg/100px-5.20_Russian_road_sign.svg.png
[8.22.1]: https://upload.wikimedia.org/wikipedia/commons/thumb/a/a5/8.22.1_Russian_road_sign.svg/40px-8.22.1_Russian_road_sign.svg.png
[8.22.2]: https://upload.wikimedia.org/wikipedia/commons/thumb/e/e6/8.22.2_Russian_road_sign.svg/40px-8.22.2_Russian_road_sign.svg.png
[8.22.3]: https://upload.wikimedia.org/wikipedia/commons/thumb/2/2d/8.22.3_Russian_road_sign.svg/40px-8.22.3_Russian_road_sign.svg.png
