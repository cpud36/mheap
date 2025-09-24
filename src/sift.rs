use crate::{ordering::Ordering, Position, hole::Hole, storage::Storage};

// The implementations of sift_up and sift_down use unsafe blocks in
// order to move an element out of the vector (leaving behind a
// hole), shift along the others and move the removed element back into the
// vector at the final location of the hole.
// The `Hole` type is used to represent this, and make sure
// the hole is filled back at the end of its scope, even on panic.
// Using a hole reduces the constant factor compared to using swaps,
// which involves twice as many moves.

/// Take an element at `pos` and move it up the heap.
///
/// Returns the new position of the element.
pub(crate) fn sift_up<S: Storage + ?Sized>(
    data: &mut S,
    pos: Position,
    ord: &impl Ordering<S::Key>,
) -> Position {
    let mut hole = Hole::new(data, pos);

    // Move up while we can
    while hole.move_up(ord) {}

    hole.into_pos()
}

/// Take an element at `pos` and move it down the heap, while its elements are larger.
///
/// Returns the new position of the element.
pub(crate) fn sift_down<S: Storage + ?Sized>(
    data: &mut S,
    pos: Position,
    ord: &impl Ordering<S::Key>,
) -> Position {
    let mut hole = Hole::new(data, pos);

    while let Some(child) = hole.upper_child_whole(ord) {
        // SAFETY: child is always different from `hole.pos`
        if unsafe { !hole.move_down(child, ord) } {
            return hole.into_pos();
        }
    }

    if let Some(child) = hole.upper_child_partial(ord) {
        // SAFETY: same as above
        unsafe {
            hole.move_down(child, ord);
        }
    }

    hole.into_pos()
}

/// Take an element at `pos` and move it all the way down the heap,
/// then sift it up to its position.
///
/// Note: This is faster when the element is known to be large / should
/// be closer to the bottom.
pub(crate) fn sift_down_to_bottom<S: Storage + ?Sized>(
    data: &mut S,
    pos: Position,
    ord: &impl Ordering<S::Key>,
) -> Position {
    let mut hole = Hole::new(data, pos);

    // unconditionally move down to the bottom
    while let Some(child) = hole.upper_child_whole(ord) {
        // SAFETY: child is always different from `hole.pos`
        unsafe {
            hole.move_to(child);
        }
    }

    if let Some(child) = hole.upper_child_partial(ord) {
        // SAFETY: same as above
        unsafe {
            hole.move_to(child);
        }
    }

    hole.into_pos()
}

/// Take an element at `pos` and move it either down, or up the heap to restore the heap invariant
///
/// Returns the new position of the element.
pub(crate) fn fixup_sift<S: Storage + ?Sized>(
    data: &mut S,
    pos: Position,
    ord: &impl Ordering<S::Key>,
) -> Position {
    let new_pos = sift_up(data, pos, ord);
    // If we sifted up, we are never going to need to sift down (in a correct heap)
    if new_pos != pos {
        return new_pos;
    }
    sift_down(data, pos, ord)
}
