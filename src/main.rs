use std::io::BufReader;
use std::fs::File;
use std::path::Path;
use std::collections::HashMap;
use log::{debug};

use structopt::StructOpt;
use serde_derive::Deserialize;

mod cli;

#[derive(Debug, Deserialize, Clone)]
struct RoadSign {
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    class: String,
}

#[derive(Debug, Deserialize, Clone)]
struct Record {
    frame: String,
    sign: RoadSign,
}

#[derive(Clone, Default, Debug)]
struct IndexItem {
    gtruth: Vec<RoadSign>,
    solutions: Vec<RoadSign>,
}

#[derive(Debug, Default, Copy, Clone)]
struct ScoreStats {
    score: f32,
    penalty: f32,
    class_scores: [f32; CLASSES_N],
    class_penalties: [f32; CLASSES_N],
}

type SignIndex = HashMap<String, IndexItem>;
type Result<T> = std::result::Result<T, Box<std::error::Error>>;
type Reader = csv::Reader<BufReader<File>>;

/// Number of classes
const CLASSES_N: usize = 2;
/// Allowed sign classes.
static ALLOWED_CLASSES: [&'static str; CLASSES_N] = [
    "5.19",
    "3.1",
];
/// False positive detection penalty
const FP_PENALTY: f32 = 3.0;

/// Checks bounding box coordinates and sign class.
fn check_sign(sign: &RoadSign) -> Result<()> {
    if !sign.x1.is_finite() {
        Err(format!("x1 is not finite: {}", sign.x1))?;
    }
    if !sign.x2.is_finite() {
        Err(format!("x2 is not finite: {}", sign.x2))?;
    }
    if sign.x1 > sign.x2 {
        Err(format!("x1 is bigger than x2: {} {}", sign.x1, sign.x2))?;
    }
    if !sign.y1.is_finite() {
        Err(format!("y1 is not finite: {}", sign.y1))?;
    }
    if !sign.y2.is_finite() {
        Err(format!("y2 is not finite: {}", sign.y2))?;
    }
    if sign.y1 > sign.y2 {
        Err(format!("y1 is bigger than y2: {} {}", sign.y1, sign.y2))?;
    }
    if !ALLOWED_CLASSES.contains(&sign.class.as_str()) {
        Err(format!("invalid sign class: {}", sign.class))?;
    }
    Ok(())
}

/// Default TSV reader.
fn default_reader(path: &Path) -> Result<Reader> {
    let gt_reader = BufReader::new(File::open(path)?);
    let rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .delimiter(b'\t')
        .from_reader(gt_reader);
    Ok(rdr)
}

/// Read ground truth file.
fn read_gt(path: &Path) -> Result<SignIndex> {
    let mut rdr = default_reader(path)?;
    let mut gt_index = SignIndex::new();
    for record in rdr.deserialize() {
        let Record { frame, sign } = record?;
        check_sign(&sign)?;
        gt_index.entry(frame)
            .or_insert_with(Default::default)
            .gtruth
            .push(sign);
    }
    Ok(gt_index)
}

/// Read solution file into index.
fn read_solution(path: &Path, index: &mut SignIndex) -> Result<()> {
    let mut rdr = default_reader(path)?;
    for record in rdr.deserialize() {
        let Record { frame, sign } = record?;
        check_sign(&sign)?;
        if let Some(item) = index.get_mut(&frame) {
            item.solutions.push(sign);
        }
    }
    Ok(())
}

/// Update score stats.
fn update_stats(mut accum: ScoreStats, new_stats: ScoreStats) -> ScoreStats {
    accum.score += new_stats.score;
    accum.penalty += new_stats.penalty;
    for i in 0..CLASSES_N {
        accum.class_scores[i] += new_stats.class_scores[i];
        accum.class_penalties[i] += new_stats.class_penalties[i];
    }
    accum
}

/// Compute Intersection Over Union for two bounding boxes.
fn compute_iou(s1: &RoadSign, s2: &RoadSign) -> f32 {
    let a1 = (s1.x2 - s1.x1)*(s1.y2 - s1.y1);
    let a2 = (s2.x2 - s2.x1)*(s2.y2 - s2.y1);

    let x1 = s1.x1.max(s2.x1);
    let x2 = s1.x2.min(s2.x2);
    let y1 = s1.y1.max(s2.y1);
    let y2 = s1.y2.min(s2.y2);
    if x1 < x2 && y1 < y2 {
        let inters = (x2 - x1)*(y2 - y1);
        inters/(a1 + a2 - inters)
    } else {
        0.0
    }
}

/// Convert IoU to score value.
fn iou2score(iou: f32) -> f32 {
    assert!(iou >= 0.5 && iou <= 1.);
    (2.*iou - 1.).sqrt().sqrt()
}

/// Find index of the given sign class.
fn find_class_idx(class: &str) -> Option<usize> {
    ALLOWED_CLASSES.iter().enumerate().find_map(|(i, &val)| {
        if val == class { Some(i) } else { None }
    })
}

/// Compute score stats for given frame.
fn compute_score(item: IndexItem) -> ScoreStats {
    let IndexItem { gtruth, solutions } = item;

    #[derive(Debug, Clone, Copy)]
    struct Hit { gt_idx: usize, sol_idx: usize, iou: f32 }

    let mut hits = vec![];

    for (gt_idx, gt_item) in gtruth.iter().enumerate() {
        for (sol_idx, sol_item) in solutions.iter().enumerate() {
            if gt_item.class != sol_item.class { continue; }
            let iou = compute_iou(gt_item, sol_item);
            if iou < 0.5 { continue; }
            hits.push(Hit { gt_idx, sol_idx, iou } )
        }
    }

    debug!("hits: {:?}", hits);

    let mut selected_hits = vec![];
    while hits.len() != 0 {
        let max = *hits.iter()
            .max_by(|Hit { iou: a, .. }, Hit { iou: b, .. }| {
                // we know that there is no NaNs
                a.partial_cmp(b).unwrap()
            })
            // vector contains at least one element
            .unwrap();
        selected_hits.push(max);
        hits.retain(|h| h.gt_idx != max.gt_idx && h.sol_idx != max.sol_idx);
    }

    debug!("selected hits: {:?}", selected_hits);

    let mut stats = ScoreStats::default();
    for hit in selected_hits.iter() {
        let score = iou2score(hit.iou);
        stats.score += score;

        let class_idx = find_class_idx(&solutions[hit.sol_idx].class)
            .expect("classes should've been filtered");
        stats.class_scores[class_idx] += score;
    }

    // find all non-selected solutions
    let leftovers: Vec<RoadSign> = solutions.into_iter()
        .enumerate()
        .filter(|(i, _)| selected_hits.iter().all(|h| h.sol_idx != *i))
        .map(|(_, v)| v)
        .collect();

    debug!("leftovers: {:?}", leftovers);

    for val in leftovers {
        stats.score -= FP_PENALTY;
        stats.penalty += FP_PENALTY;

        let class_idx = find_class_idx(&val.class)
            .expect("classes should've been filtered");
        stats.class_scores[class_idx] -= FP_PENALTY;
        stats.class_penalties[class_idx] += FP_PENALTY;
    }

    stats
}


fn main() -> Result<()> {
    let opt = cli::Cli::from_args();

    let mut index = read_gt(&opt.ground_truth)?;
    read_solution(&opt.solution, &mut index)?;

    let stats = index
        .drain()
        .map(|(_, item)| compute_score(item))
        .fold(ScoreStats::default(), update_stats);

    println!("Total score:\t{:.3}", stats.score);
    println!("Penalty:\t{:.3}", stats.penalty);
    for (s, class) in stats.class_scores.iter().zip(&ALLOWED_CLASSES) {
        println!("Score {}:\t{:.3}", class, s);
    }
    for (p, class) in stats.class_penalties.iter().zip(&ALLOWED_CLASSES) {
        println!("Penalty {}:\t{:.3}", class, p);
    }
    Ok(())
}
