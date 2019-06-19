use std::io::{self, BufReader};
use std::fs::{self, File};
use std::path::Path;
use std::collections::HashMap;
use serde_derive::Deserialize;

use crate::consts::ALLOWED_CLASSES;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
type SignIndex = HashMap<String, IndexItem>;
type Reader = csv::Reader<BufReader<File>>;

#[derive(Debug, Deserialize, Clone)]
pub struct RoadSign {
    pub xtl: f32,
    pub ytl: f32,
    pub xbr: f32,
    pub ybr: f32,
    pub class: String,
}

#[derive(Clone, Default, Debug)]
pub struct IndexItem {
    pub gtruth: Vec<RoadSign>,
    pub solutions: Vec<RoadSign>,
}

#[derive(Debug, Deserialize, Clone)]
struct SolutionRecord {
    frame: String,
    xtl: f32,
    ytl: f32,
    xbr: f32,
    ybr: f32,
    class: String,
}

impl SolutionRecord {
    fn split(self) -> (String, RoadSign) {
        (self.frame, RoadSign {
            xtl: self.xtl,
            ytl: self.ytl,
            xbr: self.xbr,
            ybr: self.ybr,
            class: self.class,
        })
    }
}

/// Checks bounding box coordinates and sign class.
fn check_sign(sign: &RoadSign) -> Result<()> {
    if !sign.xtl.is_finite() {
        Err(format!("xtl is not finite: {}", sign.xtl))?;
    }
    if !sign.xbr.is_finite() {
        Err(format!("xbr is not finite: {}", sign.xbr))?;
    }
    if sign.xtl > sign.xbr {
        Err(format!("xtl is bigger than xbr: {} {}", sign.xtl, sign.xbr))?;
    }
    if !sign.ytl.is_finite() {
        Err(format!("ytl is not finite: {}", sign.ytl))?;
    }
    if !sign.ybr.is_finite() {
        Err(format!("ybr is not finite: {}", sign.ybr))?;
    }
    if sign.ytl > sign.ybr {
        Err(format!("ytl is bigger than ybr: {} {}", sign.ytl, sign.ybr))?;
    }
    if !ALLOWED_CLASSES.contains(&sign.class.as_str()) {
        Err(format!("invalid sign class: {}", sign.class))?;
    }
    Ok(())
}

/// Ground truth TSV reader.
fn gt_reader(path: &Path) -> Result<Reader> {
    let reader = BufReader::new(File::open(path)?);
    let rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .delimiter(b'\t')
        .from_reader(reader);
    Ok(rdr)
}

/// Convert road sign to one used for online stage.
///
/// Returns `None` if sign is not used in online stage.
fn convert_sign(mut s: RoadSign) -> Option<RoadSign> {
    if s.class.starts_with("4.1") { s.class = "4.1".to_string(); }
    if s.class.starts_with("4.2") { s.class = "4.2".to_string(); }
    if s.class.starts_with("5.19") { s.class = "5.19".to_string(); }
    if s.class.starts_with("8.22") { s.class = "8.22".to_string(); }
    if ALLOWED_CLASSES.contains(&s.class.as_str()) {
        Some(s)
    } else {
        None
    }
}

/// Read ground truth and solution files.
pub fn read_files(gt_path: &Path, sol_path: &Path) -> Result<SignIndex> {
    let mut index = SignIndex::new();

    for entry in fs::read_dir(gt_path)? {
        let path = entry?.path();
        let set_name = path.file_name()
            .and_then(|v| v.to_str())
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput,
                "failed to derive set name from directory name"))?;
        if !path.is_dir() { continue }
        for entry in fs::read_dir(&path)? {
            let path = &entry?.path();
            if path.extension().map(|v| v != "tsv").unwrap_or(true) {
                continue;
            }
            let mut gt_rdr = gt_reader(path)?;
            let frame = path.file_stem()
                .and_then(|v| v.to_str())
                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput,
                    "failed to get frame name"))?;
            let frame = [set_name, frame].join("/");

            let entry = &mut index.entry(frame)
                    .or_insert_with(Default::default)
                    .gtruth;
            for record in gt_rdr.deserialize() {
                let sign = match convert_sign(record?) {
                    Some(s) => s,
                    None => continue,
                };
                check_sign(&sign)?;
                entry.push(sign);
            }
        }
    }

    let reader = BufReader::new(File::open(sol_path)?);
    let mut sol_rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .delimiter(b'\t')
        .from_reader(reader);
    for record in sol_rdr.deserialize() {
        let record: SolutionRecord = record?;
        let (frame, sign) = record.split();
        check_sign(&sign)?;
        // ignore signs with area smaller than 100 pixels
        if (sign.xbr - sign.xtl)*(sign.xbr - sign.xtl) < 100. {
            continue;
        }
        if let Some(item) = index.get_mut(&frame) {
            item.solutions.push(sign);
        }
    }
    Ok(index)
}
