/// Number of classes
pub const CLASSES_N: usize = 2;
/// Allowed sign classes.
pub static ALLOWED_CLASSES: [&'static str; CLASSES_N] = [
    "5.19",
    "3.1",
];
/// False positive detection penalty
pub const FP_PENALTY: f32 = 3.0;
