use log::debug;

use crate::consts::{CLASSES_N, ALLOWED_CLASSES, FP_PENALTY, IOU_BOT, IOU_TOP};
use crate::read::{RoadSign, IndexItem};

/// Solution score and statistics.
#[derive(Debug, Default, Copy, Clone)]
pub struct ScoreStats {
    pub score: f32,
    pub penalty: f32,
    pub class_scores: [f32; CLASSES_N],
    pub class_penalties: [f32; CLASSES_N],
}

impl ScoreStats {
    /// Update score stats.
    pub fn update(&mut self, new_stats: ScoreStats) {
        self.score += new_stats.score;
        self.penalty += new_stats.penalty;
        for i in 0..CLASSES_N {
            self.class_scores[i] += new_stats.class_scores[i];
            self.class_penalties[i] += new_stats.class_penalties[i];
        }
    }
}

/// Compute Intersection Over Union for two bounding boxes.
fn compute_iou(s1: &RoadSign, s2: &RoadSign) -> f32 {
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
        println!("{:?}", iou);
        panic!("expected IoU to be bigger than IOU_BOT");
    }
}

/// Find index of the given sign class.
fn find_class_idx(class: &str) -> Option<usize> {
    ALLOWED_CLASSES.iter().enumerate().find_map(|(i, &val)| {
        if val == class { Some(i) } else { None }
    })
}

/// Compute score stats for given frame.
pub fn compute_score(item: IndexItem) -> ScoreStats {
    let IndexItem { gtruth, solutions } = item;
    println!("{:#?}\n\n{:#?}", gtruth, solutions);

    #[derive(Debug, Clone, Copy)]
    struct Hit { gt_idx: usize, sol_idx: usize, iou: f32 }

    let mut hits = vec![];

    for (gt_idx, gt_item) in gtruth.iter().enumerate() {
        for (sol_idx, sol_item) in solutions.iter().enumerate() {
            if gt_item.class != sol_item.class { continue; }
            let iou = compute_iou(gt_item, sol_item);
            if iou < IOU_BOT { continue; }
            hits.push(Hit { gt_idx, sol_idx, iou } )
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
