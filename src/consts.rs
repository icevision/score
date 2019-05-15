/// Number of classes
pub const CLASSES_N: usize = 10;
/// Allowed sign classes.
pub static ALLOWED_CLASSES: [&'static str; CLASSES_N] = [
    "2.1",
    "2.4",
    "3.1",
    "3.24",
    "3.27",
    "4.1",
    "4.2",
    "5.19",
    "5.20",
    "8.22",
];
/// False positive detection penalty
pub const FP_PENALTY: f32 = 3.0;
