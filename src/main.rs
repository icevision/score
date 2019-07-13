use structopt::StructOpt;

mod cli;
mod consts;
mod read;
mod score;

use score::{ScoreStats, update_score};
use read::Result;

fn main() -> Result<()> {
    let opt = cli::Cli::from_args();

    let mut index = read::read_files(&opt.ground_truth, &opt.solution)?;

    let mut stats = ScoreStats::default();
    let mut keys: Vec<String> = index.iter()
        .filter(|v| v.1.solutions.len() != 0)
        .map(|v| v.0.clone())
        .collect();
    keys.sort_unstable();
    for key in keys {
        let val = index.remove(&key).expect("key is valid");
        if opt.verbose {
            println!("\nframe: {}", key);
        }
        update_score(&mut stats, val, opt.verbose);
    }

    if opt.verbose {
        println!("\n===========================\n");
    }

    println!("Total score:\t{:.3}", stats.score);
    println!("Total penalty:\t{:.3}", stats.penalty);

    let mut per_class: Vec<_> = stats.per_class.iter().collect();

    // we know that we can't have NaNs here
    per_class.sort_unstable_by_key(|v| -((v.1).0*1000.) as i64);

    println!("Per class results:");
    println!("Class\tScore\tPenalty");
    for (class, (score, penalty)) in per_class {
        println!("{}\t{:.3}\t{:.3}", class, score, penalty);
    }

    Ok(())
}
