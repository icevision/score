use structopt::StructOpt;
use std::path::PathBuf;

#[derive(StructOpt)]
#[structopt(name = "icevision-score",
    about = "IceVision competition scoring software")]
pub struct Cli {
    /// Path to a directory with ground truth TSV files.
    pub ground_truth: PathBuf,
    /// Path to a solution TSV file.
    pub solution: PathBuf,
}
