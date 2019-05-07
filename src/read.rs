use std::io::BufReader;
use std::fs::File;
use std::path::Path;
use std::collections::HashMap;
use serde_derive::Deserialize;

use crate::consts::ALLOWED_CLASSES;

pub type Result<T> = std::result::Result<T, Box<std::error::Error>>;
type SignIndex = HashMap<String, IndexItem>;
type Reader = csv::Reader<BufReader<File>>;

#[derive(Debug, Deserialize, Clone)]
pub struct RoadSign {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
    pub class: String,
}

#[derive(Clone, Default, Debug)]
pub struct IndexItem {
    pub gtruth: Vec<RoadSign>,
    pub solutions: Vec<RoadSign>,
}

#[derive(Debug, Deserialize, Clone)]
struct Record {
    frame: String,
    sign: RoadSign,
}

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

/// Read ground truth and solution files.
pub fn read_files(gt_path: &Path, sol_path: &Path) -> Result<SignIndex> {
    let mut index = SignIndex::new();

    let mut gt_rdr = default_reader(gt_path)?;
    for record in gt_rdr.deserialize() {
        let Record { frame, sign } = record?;
        check_sign(&sign)?;
        index.entry(frame)
            .or_insert_with(Default::default)
            .gtruth
            .push(sign);
    }

    let mut sol_rdr = default_reader(sol_path)?;
    for record in sol_rdr.deserialize() {
        let Record { frame, sign } = record?;
        check_sign(&sign)?;
        if let Some(item) = index.get_mut(&frame) {
            item.solutions.push(sign);
        }
    }
    Ok(index)
}
