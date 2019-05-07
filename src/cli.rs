use structopt::StructOpt;
use std::path::PathBuf;

#[derive(StructOpt)]
#[structopt(name = "convert",
    about = "IceVision competition scoring software")]
pub struct Cli {
    pub ground_truth: PathBuf,
    pub solution: PathBuf,
}
