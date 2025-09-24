use crate::{Position, storage::Storage};

pub(crate) fn root<S: Storage + ?Sized>(data: &S) -> Option<Position> {
    (data.len() != 0).then_some(0)
}

/// Returns an item parent node
///
/// It is guaranteed that the parent is different from the argument
pub(crate) fn parent<S: Storage + ?Sized>(_data: &S, pos: Position) -> Option<Position> {
    // SAFETY: consider `k > 0`. Then there are 3 cases:
    // case pos = 2k + 1:
    //    parent = (2k - 1) / 2 = k;
    // case pos = 2k + 2:
    //    parent = (2k + 1) / 2 = k;
    // case pos = 0;
    //    we return None
    // Since 2k > k, we never return the pos itself
    Some(pos.checked_sub(1)? / 2)
}

/// Returns nth child of a node
///
/// It is guaranteed that the child is different from the argument
pub(crate) fn child<S: Storage + ?Sized>(
    data: &S,
    pos: Position,
    index: usize,
) -> Option<Position> {
    assert!(index < 2);
    // FIXME: this expression could overflow if T is a ZST
    let child = 2 * pos + 1 + index;
    // SAFETY: for any `pos` we have `2 * pos >= pos`, and `1 + index > 0`, so `child > pos`
    (child < data.len()).then_some(child)
}

/// Selects a node, or its next sibling, based on the condition
pub(crate) fn select_sibling<S: Storage + ?Sized>(
    _data: &S,
    pos: Position,
    cond: bool,
) -> Option<Position> {
    Some(pos + (cond as usize))
}

/// Checks if a node has all children
pub(crate) fn is_whole_node<S: Storage + ?Sized>(data: &S, pos: Position) -> bool {
    child(data, pos, 1).is_some()
}

/// Checks if a node has all children
pub(crate) fn nchildren<S: Storage + ?Sized>(data: &S, pos: Position) -> usize {
    let first = 2 * pos + 1;
    let len = data.len();
    let s = len.saturating_sub(first);
    s.min(2)
}

pub(crate) fn children<S: Storage + ?Sized>(
    data: &S,
    pos: Position,
) -> impl Iterator<Item = Position> {
    let n = nchildren(data, pos);
    (0..n).map(move |index| child(data, pos, index).unwrap())
}

pub(crate) fn rebuild_range<S: Storage + ?Sized>(data: &S) -> std::ops::Range<Position> {
    let len = data.len();
    let n = len / 2;
    0..n
}

/// Whether it is better to rebuild the whole heap or to rebuild only the tail
pub(crate) fn better_to_rebuild<S: Storage + ?Sized>(data: &S, start: Position) -> bool {
    let len = data.len();
    let tail_len = len - start;

    // `rebuild` takes O(self.len()) operations
    // and about 2 * self.len() comparisons in the worst case
    // while repeating `sift_up` takes O(tail_len * log(start)) operations
    // and about 1 * tail_len * log_2(start) comparisons in the worst case,
    // assuming start >= tail_len. For larger heaps, the crossover point
    // no longer follows this reasoning and was determined empirically.
    if start < tail_len {
        true
    } else if len <= 2048 {
        2 * len < tail_len * log2_fast(start)
    } else {
        2 * len < tail_len * 11
    }
}

fn log2_fast(x: usize) -> usize {
    (usize::BITS - x.leading_zeros() - 1) as usize
}

#[cfg(test)]
mod tests {

    #[test]
    fn children() {
        assert_eq!(super::nchildren([0u32; 3].as_slice(), 0), 2);
        assert_eq!(super::nchildren([0u32; 3].as_slice(), 1), 0);
        assert_eq!(super::nchildren([0u32; 4].as_slice(), 1), 1);
        assert_eq!(super::nchildren([0u32; 5].as_slice(), 1), 2);
        assert_eq!(super::nchildren([0u32; 6].as_slice(), 1), 2);
    }
}
