use structopt::StructOpt;

mod cli;
mod consts;
mod read;
mod score;

use consts::ALLOWED_CLASSES;
use score::{ScoreStats, compute_score};
use read::Result;

fn main() -> Result<()> {
    let opt = cli::Cli::from_args();

    let index = read::read_files(&opt.ground_truth, &opt.solution)?;

    let mut stats = ScoreStats::default();
    let iter = index.into_iter().map(|v| v.1).filter(|v| v.solutions.len() != 0);
    for item in iter {
        stats.update(compute_score(item));
    }

    println!("Total score:\t{:.3}", stats.score);
    println!("Total penalty:\t{:.3}", stats.penalty);
    for (s, class) in stats.class_scores.iter().zip(&ALLOWED_CLASSES) {
        println!("Score {}:\t{:.3}", class, s);
    }
    for (p, class) in stats.class_penalties.iter().zip(&ALLOWED_CLASSES) {
        println!("Penalty {}:\t{:.3}", class, p);
    }
    Ok(())
}
