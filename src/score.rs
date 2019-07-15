use crate::consts::{FP_PENALTY, IOU_BOT, IOU_TOP};
use crate::read::{SignAnnotation, SignDetection, Bbox, SignClass, IndexItem};
use std::collections::HashMap;

/// Solution score and statistics.
#[derive(Debug, Default, Clone)]
pub struct ScoreStats {
    pub score: f32,
    pub penalty: f32,
    // first float is score, second penalty
    pub per_class: HashMap<SignClass, (f32, f32)>,
}
/// Compute Intersection Over Union for two bounding boxes.
fn compute_iou(s1: Bbox, s2: Bbox) -> f32 {
    let a1 = (s1.xbr - s1.xtl)*(s1.ybr - s1.ytl);
    let a2 = (s2.xbr - s2.xtl)*(s2.ybr - s2.ytl);

    let xtl = s1.xtl.max(s2.xtl);
    let xbr = s1.xbr.min(s2.xbr);
    let ytl = s1.ytl.max(s2.ytl);
    let ybr = s1.ybr.min(s2.ybr);
    if xtl < xbr && ytl < ybr {
        let inters = (xbr - xtl)*(ybr - ytl);
        inters/(a1 + a2 - inters)
    } else {
        0.0
    }
}

/// Convert IoU to score.
fn iou2score(iou: f32) -> f32 {
    if iou > IOU_TOP {
        1.0
    } else if iou >= IOU_BOT {
        ((iou - IOU_BOT)/(IOU_TOP - IOU_BOT)).sqrt().sqrt()
    } else {
        panic!("expected IoU to be bigger than IOU_BOT");
    }
}

/// Compute k1 in percents
fn compute_k1(gt: SignClass, det: SignClass) -> Option<i32> {
    use SignClass::*;
    match (gt, det) {
        (Double(_, _), Double(_, _))
            | (Triple(_, _, _), Triple(_, _, _))
            if gt == det
            => Some(0),
        (Triple(g1, g2, _), Double(d1, d2))
            if g1 == d1 && g2 == d2
            => Some(-20),
        (Double(g1, _), Single(d1))
            | (Triple(g1, _, _), Single(d1))
            if g1 == d1
            => Some(-70),
        (Single(8), Single(8))
            | (Single(8), Double(8, _))
            | (Single(8), Triple(8, _, _))
            => Some(0),
        (Single(8), _) => None,
        (Single(_), _) => panic!("unexpected annotation class: {:?}", gt),
        (Na, _) => Some(0),
        (_, Na) => None, // detections should not use NA class
        _ => None,
    }
}

fn compute_k2(gt: &SignAnnotation, det: &SignDetection) -> i32 {
    fn normalize(s: &str) -> String {
        s.chars()
            .filter(|&c| c != ' ')
            .map(|c| if c == ',' { '.' } else { c })
            .collect::<String>()
            .to_lowercase()
    }

    match (&gt.data, &det.data) {
        (Some(s1), Some(s2)) => if normalize(s1) == normalize(s2) {
            200
        } else {
            -50
        },
        (None, Some(_)) => -50,
        (Some(_), None) | (None, None) => 0,
    }
}

fn compute_k3(gt: &SignAnnotation, det: &SignDetection) -> i32 {
    match (gt.temporary, det.temporary) {
        (true, Some(true)) =>  100,
        (false, Some(true)) | (true, Some(false)) => -50,
        _ => 0,
    }
}

/// Compute score stats for given frame.
pub fn update_score(stats: &mut ScoreStats, item: IndexItem, verbose: bool) {
    let IndexItem { gtruth, solutions } = item;

    #[derive(Debug, Clone, Copy)]
    struct Hit {
        gt_idx: usize, sol_idx: usize,
        iou: f32, score: f32,
        s: f32, k1: i32, k2: i32, k3: i32,
    }

    let mut hits = vec![];

    for (gt_idx, gt_item) in gtruth.iter().enumerate() {
        for (sol_idx, sol_item) in solutions.iter().enumerate() {
            let k1 = match compute_k1(gt_item.class, sol_item.class) {
                Some(k) => k,
                None => continue,
            };
            let iou = compute_iou(gt_item.bbox, sol_item.bbox);
            if iou < IOU_BOT { continue; }
            let s = if gt_item.class != SignClass::Na {
                iou2score(iou)
            } else {
                0.0
            };
            let k2 = compute_k2(gt_item, sol_item);
            let k3 = compute_k3(gt_item, sol_item);;
            let score = if k1 + k2 + k3 > -100 {
                let k = ((100 + k1 + k2 + k3) as f32)/100.;
                k*s
            } else {
                0.0
            };
            hits.push(Hit { gt_idx, sol_idx, iou, score, s, k1, k2, k3 } )
        }
    }

    let mut selected_hits = vec![];
    while hits.len() != 0 {
        // find hit with a maximum IoU
        let max = *hits.iter()
            .max_by(|Hit { iou: a, .. }, Hit { iou: b, .. }| {
                // we know that there are no NaNs
                a.partial_cmp(b).unwrap()
            })
            // vector contains at least one element
            .unwrap();
        selected_hits.push(max);
        // remove hits which use selected ground truth and solution
        hits.retain(|h| h.gt_idx != max.gt_idx && h.sol_idx != max.sol_idx);
    }

    if verbose {
        println!("score\txtl\tytl\txbr\tybr\tclass\ts\tk1\tk2\tk3");
        for (i, s) in solutions.iter().enumerate() {
            let b = s.bbox;
            match selected_hits.iter().find(|v| v.sol_idx == i) {
                Some(v) => println!(
                    "{:.3}\t{}\t{}\t{}\t{}\t{}\t{:.3}\t{}\t{}\t{}",
                    v.score, b.xtl, b.ytl, b.xbr, b.ybr, s.class,
                    v.s, v.k1, v.k2, v.k3,
                ),
                None => println!(
                    "{:.3}\t{}\t{}\t{}\t{}\t{:.3}\t-\t-\t-\t-",
                    -FP_PENALTY, b.xtl, b.ytl, b.xbr, b.ybr, s.class,
                ),
            }
        }
    }

    for hit in selected_hits.iter() {
        stats.score += hit.score;
        let class = solutions[hit.sol_idx].class.truncate();
        let entry = stats.per_class.entry(class).or_insert((0.0, 0.0));
        entry.0 += hit.score;
    }

    // find all non-selected solutions
    let leftovers: Vec<SignDetection> = solutions.into_iter()
        .enumerate()
        .filter(|(i, _)| selected_hits.iter().all(|h| h.sol_idx != *i))
        .map(|(_, v)| v)
        .collect();

    for val in leftovers {
        stats.score -= FP_PENALTY;
        stats.penalty += FP_PENALTY;

        let class = val.class.truncate();
        let entry = stats.per_class.entry(class).or_insert((0.0, 0.0));
        entry.0 -= FP_PENALTY;
        entry.1 += FP_PENALTY;
    }
}
