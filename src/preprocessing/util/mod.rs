use std::collections::HashMap;

pub mod document_frequency;
pub mod term_frequency;

pub(super) fn merge_counts(
    mut left: HashMap<String, usize>,
    mut right: HashMap<String, usize>,
) -> HashMap<String, usize> {
    // Merge the smaller map into the larger one to reduce insertions.
    if left.len() < right.len() {
        std::mem::swap(&mut left, &mut right);
    }
    for (word, count) in right {
        *left.entry(word).or_insert(0) += count;
    }
    left
}
