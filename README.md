# icevision-score [![Build Status](https://travis-ci.org/icevision/score.svg?branch=master)](https://travis-ci.org/icevision/score) [![dependency status](https://deps.rs/repo/github/icevision/score/status.svg)](https://deps.rs/repo/github/icevision/score)

Scoring software used in the IceVision competition.

## Input file formats

For ground truth we use the same format [annotations] repository.

For solutions we use a slightly different TSV (tab-separated values) file format. It contains all detected traffic signs of given classes on provided frame sequences. A solution file must contain header and the following fields:
- `frame`: sequence + frame number, e.g. `2018-02-13_1418_left/000032`.
- `xtl`, `ytl`, `xbr`, `ybr`: bounding box coordinates, integer or float. Note: `xtl` must be bigger than `xbr` and `ytl` must be bigger than `ybr`
- `class`: traffic sign code. Valid values are: `2.1`, `2.4`, `3.1`, `3.24`, `3.27`, `4.1`, `4.2`, `5.19`, `5.20`, `8.22`.

Some examples can be found in [`examples/`] folder.

[`examples/`]: https://github.com/icevision/score/tree/master/src

## Scoring methodology

During online stage participants have to detect the following traffic signs:

<details>
  <summary>Click to expand table</summary>

| Code | Image   | Description |
| -----|:-------:| :----------:|
| 2.1  | ![2.1]  | Main road |
| 2.4  | ![2.4]  | Yield road |
| 3.1  | ![3.1]  | No entry |
| 3.24 | ![3.24] | Maximum speed limit |
| 3.27 | ![3.27] | No stopping |
| 4.1 | ![4.1.1] ![4.1.2] ![4.1.3] <br/>![4.1.4] ![4.1.5] ![4.1.6] | Proceed in the given direction |
| 4.2 | ![4.2.1] ![4.2.2] ![4.2.3] | Pass on the given side |
| 5.19 | ![5.19.1] ![5.19.2] | Pedestrian crossing |
| 5.20 | ![5.20] | Road bump |
| 8.22 | ![8.22.1] ![8.22.2] ![8.22.3] | Obstacle |
</details>

Detection is considered successful if [IoU] is bigger or equal to 0.5 and bounding box has a correct class code. If sign is detected twice, then detection with a smallest IoU will be counted as a false positive. Each false positive or incorrect detection results in penalty equal to 2 points.

If IoU of a true positive detection is bigger than 0.85, it results in adding 1 point to final result. Otherwise points are calculated as `((IoU - 0.5)/0.35)^0.25`.

The final score is computed as sum of all true positive points minus all penalties.

For exact algorithm, please, refer to the code.

[IoU]: https://en.wikipedia.org/wiki/Jaccard_index

## Building
Scoring software is written in Rust, so you'll need to grab a
[Rust installation] in order to compile it. In general, we use the latest
stable release of the Rust compiler, but older versions may work as well.

```
$ git clone https://github.com/BurntSushi/ripgrep
$ cd ripgrep
$ cargo build --release
$ ./target/release/icevision-score --help
convert 0.1.0
Artyom Pavlov <newpavlov@gmail.com>
IceVision competition scoring software

USAGE:
    icevision-score <ground_truth> <solution>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

ARGS:
    <ground_truth>    Path to a directory with ground truth TSV files.
    <solution>        Path to a solution TSV file.
```


[Rust installation]: https://www.rust-lang.org/

## Usage example
For ground truth we use some files from [annotations] repository.
```
$ ./icevision-score examples/ground_truth/ examples/good.tsv
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
