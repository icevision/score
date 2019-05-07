use structopt::StructOpt;
use std::path::PathBuf;

#[derive(StructOpt)]
#[structopt(name = "convert",
    about = "IceVision competition scoring software")]
pub struct Cli {
    /// Path to ground truth TSV file
    pub ground_truth: PathBuf,
    /// Path to solution TSV file
    pub solution: PathBuf,
}
