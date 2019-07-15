use std::io::{self, BufReader};
use std::fs::{self, File};
use std::path::Path;
use std::collections::HashMap;
use serde_derive::Deserialize;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
type SignIndex = HashMap<String, IndexItem>;
type Reader = csv::Reader<BufReader<File>>;

mod sign_class {
    use serde::{self, de, Deserialize, Deserializer};
    use std::fmt;

    #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
    pub enum SignClass {
        Single(u8),
        Double(u8, u8),
        Triple(u8, u8, u8),
        Na,
    }

    impl SignClass {
        /// Truncate third integer if present
        pub fn truncate(self) -> Self {
            match self {
                SignClass::Triple(a, b, _) => SignClass::Double(a, b),
                v => v,
            }
        }
    }

    impl fmt::Display for SignClass {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            use SignClass::*;
            match self {
                Single(a) => write!(f, "{}", a),
                Double(a, b) => write!(f, "{}.{}", a, b),
                Triple(a, b, c) => write!(f, "{}.{}.{}", a, b, c),
                Na => write!(f, "NA"),
            }
        }
    }

    pub fn deserialize<'de, D>(deserializer: D)
        -> Result<SignClass, D::Error>
        where D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        if s == "NA" {
            return Ok(SignClass::Na)
        }
        let res: Vec<u8> = s
            .split('.')
            .map(std::str::FromStr::from_str)
            .collect::<Result<_, _>>()
            .map_err(|_| de::Error::custom("invalid sign class"))?;
        match res.as_slice() {
            [a] => Ok(SignClass::Single(*a)),
            [a, b] => Ok(SignClass::Double(*a, *b)),
            [a, b, c] => Ok(SignClass::Triple(*a, *b, *c)),
            _ => Err(de::Error::custom("invalid sign class")),
        }
    }
}

pub use sign_class::SignClass;

#[derive(Debug, Deserialize, Clone, Copy)]
pub struct Bbox {
    pub xtl: f32,
    pub ytl: f32,
    pub xbr: f32,
    pub ybr: f32,
}

impl Bbox {
    fn check(&mut self) -> Result<()> {
        use std::mem::swap;

        if !self.xtl.is_finite() {
            Err(format!("xtl is not finite: {}", self.xtl))?;
        }
        if !self.xbr.is_finite() {
            Err(format!("xbr is not finite: {}", self.xbr))?;
        }
        if self.xtl > self.xbr {
            swap(&mut self.xtl, &mut self.xbr);
        }
        if !self.ytl.is_finite() {
            Err(format!("ytl is not finite: {}", self.ytl))?;
        }
        if !self.ybr.is_finite() {
            Err(format!("ybr is not finite: {}", self.ybr))?;
        }
        if self.ytl > self.ybr {
            swap(&mut self.ytl, &mut self.ybr);
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct SignAnnotation {
    #[serde(flatten)]
    pub bbox: Bbox,
    #[serde(with = "sign_class")]
    pub class: SignClass,
    pub temporary: bool,
    pub occluded: bool,
    pub data: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SignDetection {
    #[serde(flatten)]
    pub bbox: Bbox,
    #[serde(with = "sign_class")]
    pub class: SignClass,
    pub temporary: Option<bool>,
    pub data: Option<String>,
}

#[derive(Clone, Default, Debug)]
pub struct IndexItem {
    pub gtruth: Vec<SignAnnotation>,
    pub solutions: Vec<SignDetection>,
}

#[derive(Debug, Deserialize, Clone)]
struct SolutionRecord {
    pub frame: String,
    #[serde(flatten)]
    pub bbox: Bbox,
    #[serde(with = "sign_class")]
    pub class: SignClass,
    pub temporary: Option<bool>,
    pub data: Option<String>,
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
            for sign in gt_rdr.deserialize() {
                let mut sign: SignAnnotation = sign?;
                sign.bbox.check()?;
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
        let SolutionRecord { frame, bbox, class, temporary, data } = record?;
        let mut sign = SignDetection { bbox, class, temporary, data };
        sign.bbox.check()?;
        let bbox = sign.bbox;
        // ignore signs with an area smaller than 100 pixels
        if (bbox.xbr - bbox.xtl)*(bbox.xbr - bbox.xtl) < 100. {
            continue;
        }
        if let Some(item) = index.get_mut(&frame) {
            item.solutions.push(sign);
        }
    }
    Ok(index)
}
