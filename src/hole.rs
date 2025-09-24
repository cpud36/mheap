use std::mem::ManuallyDrop;

use crate::{Position, ordering::Ordering, storage::Storage, tree};

/// Hole represents a hole in a slice i.e., an index without valid value
/// (because it was moved from or duplicated).
/// In drop, `Hole` will restore the slice by filling the hole
/// position with the value that was originally removed.
///
/// # Safety
///
/// Most unsafe functions require that some indexes must be valid. That means:
///
/// * `index` is within the data slice
/// * `index` is not equal to `pos`
pub(crate) struct Hole<'a, S: Storage + ?Sized> {
    data: &'a mut S,
    elt: ManuallyDrop<S::Slot>,
    pos: Position,
}

impl<S: Storage + ?Sized> Drop for Hole<'_, S> {
    #[inline]
    fn drop(&mut self) {
        // SAFETY: we have a hole at `self.pos` by type invariant
        //         and we never drop the `self.elt`
        unsafe {
            self.data.store(self.pos, &mut self.elt);
        }
    }
}

impl<'a, S: Storage + ?Sized> Hole<'a, S> {
    /// Creates a new `Hole` at index `pos`.
    pub(crate) fn new(data: &'a mut S, pos: Position) -> Self {
        // SAFETY: by safety requirements on [`Storage`] trait
        //         the data does not have holes by default.
        //         We restore this hole in our Drop implementation.
        let elt = unsafe { data.load(pos) };
        Hole { data, elt, pos }
    }

    pub(crate) fn into_pos(self) -> Position {
        let pos = self.pos;
        drop(self);
        pos
    }

    /// Returns a reference to the element removed.
    fn element(&self) -> &S::Key {
        S::slot_key(&self.elt)
    }

    /// requires `index != pos`
    pub(crate) unsafe fn move_down(
        &mut self,
        index: Position,
        ord: &impl Ordering<S::Key>,
    ) -> bool {
        if ord.should_sift_down(&self.element(), &self.data.get_key(index)) {
            // SAFETY: guaranteed by the caller
            unsafe { self.move_to(index) };
            true
        } else {
            false
        }
    }

    pub(crate) fn move_up(&mut self, ord: &impl Ordering<S::Key>) -> bool {
        let Some(parent) = tree::parent(self.data, self.pos) else {
            return false;
        };
        if ord.should_sift_up(&self.element(), &self.data.get_key(parent)) {
            // SAFETY: parent != pos by safety requirements on [`Storage`] trait
            unsafe { self.move_to(parent) };
            true
        } else {
            false
        }
    }

    pub(crate) fn upper_child_whole(&self, ord: &impl Ordering<S::Key>) -> Option<Position> {
        if !tree::is_whole_node(self.data, self.pos) {
            return None;
        }

        let first = tree::child(self.data, self.pos, 0).unwrap();
        let second = tree::child(self.data, self.pos, 1).unwrap();
        let cond = ord.select_upper(&self.data.get_key(first), &self.data.get_key(second));
        Some(
            if let Some(child) = tree::select_sibling(self.data, first, cond) {
                child
            } else {
                if cond { first } else { second }
            },
        )
    }

    pub(crate) fn upper_child_partial(&self, ord: &impl Ordering<S::Key>) -> Option<Position> {
        let mut children = tree::children(self.data, self.pos);
        let mut max = children.next()?;
        for child in children {
            if ord.select_upper(&self.data.get_key(max), &self.data.get_key(child)) {
                max = child;
            }
        }
        Some(max)
    }

    /// Move hole to new location
    ///
    /// # Safety
    ///
    /// `index != pos`
    pub(crate) unsafe fn move_to(&mut self, index: Position) {
        // SAFETY: we have a hole at `self.pos` and `index != self.pos`
        //         so there is no hole at `index`
        unsafe {
            debug_assert!(index != self.pos);
            self.data.move_element(index, self.pos);
            self.pos = index;
        }
    }
}
