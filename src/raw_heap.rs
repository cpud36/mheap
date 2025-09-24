use crate::{sift, storage::Storage, tree, ordering::Ordering, Position};

pub trait RawHeap: Storage {
    /// Take an element at `pos` and move it up the heap.
    ///
    /// Returns the new position of the element.
    fn sift_up(&mut self, pos: Position, ord: &impl Ordering<Self::Key>) -> Position {
        sift::sift_up(self, pos, ord)
    }

    /// Take an element at `pos` and move it down the heap, while its elements are larger.
    ///
    /// Returns the new position of the element.
    fn sift_down(&mut self, pos: Position, ord: &impl Ordering<Self::Key>) -> Position {
        sift::sift_down(self, pos, ord)
    }

    /// Take an element at `pos` and move it all the way down the heap (ignores the element key/value)
    fn sift_down_to_bottom(&mut self, pos: Position, ord: &impl Ordering<Self::Key>) -> Position {
        sift::sift_down_to_bottom(self, pos, ord)
    }

    /// Restore heap invariant by moving the element up or down the heap
    ///
    /// Returns the new position of the element.
    fn fixup_sift(&mut self, pos: Position, ord: &impl Ordering<Self::Key>) -> Position {
        sift::fixup_sift(self, pos, ord)
    }

    /// Specialized version of [`Self::fixup_sift`] that is faster when the element is known to be close to the bottom of the heap
    fn fixup_sift_to_bottom(&mut self, pos: Position, ord: &impl Ordering<Self::Key>) -> Position {
        let pos = self.sift_down_to_bottom(pos, ord);
        self.sift_up(pos, ord)
    }

    fn peek(&self) -> Option<&Self::Item> {
        Some(self.get(tree::root(self)?))
    }

    fn peek_mut(&mut self) -> Option<PeekMut<'_, Self>> {
        PeekMut::new(self)
    }

    fn pop_swap(
        &mut self,
        mut last_item: Self::Item,
        ord: &impl Ordering<Self::Key>,
    ) -> Self::Item {
        if let Some(pos) = tree::root(self) {
            std::mem::swap(self.get_mut(pos), &mut last_item);
            self.fixup_sift_to_bottom(pos, ord);
        }
        last_item
    }

    fn rebuild(&mut self, ord: &impl Ordering<Self::Key>) {
        for i in tree::rebuild_range(self).rev() {
            self.sift_down(i, ord);
        }
    }

    fn rebuild_tail(&mut self, start: Position, ord: &impl Ordering<Self::Key>) {
        if start == self.len() {
            return;
        }
        if tree::better_to_rebuild(self, start) {
            self.rebuild(ord);
        } else {
            assert!(start < self.len());
            for i in start..self.len() {
                self.sift_up(i, ord);
            }
        }
    }
}

impl<S: Storage + ?Sized> RawHeap for S {}

/// Structure wrapping a mutable reference to the top item on a [`RawHeap`].
///
/// This `struct` is created by the [`peek_mut`] method on [`RawHeap`]. See
/// its documentation for more.
///
/// [`peek_mut`]: RawHeap::peek_mut
pub struct PeekMut<'a, S: RawHeap + ?Sized> {
    // Invariant: heap is not empty
    heap: &'a mut S,
    sift: bool,
}

impl<'a, S: RawHeap + ?Sized> Drop for PeekMut<'a, S> {
    fn drop(&mut self) {
        if cfg!(debug_assertions) && self.sift {
            if !std::thread::panicking() {
                panic!("PeekMut must be restored before dropping");
            }
        }
    }
}

impl<'a, S: RawHeap + ?Sized> PeekMut<'a, S> {
    /// The heap must have a root node (i.e. not empty)
    fn new(heap: &'a mut S) -> Option<Self> {
        if heap.is_empty() {
            return None;
        }
        Some(Self { heap, sift: false })
    }

    fn assert_invariant(&self) {
        debug_assert!(!self.heap.is_empty());
        if self.heap.is_empty() {
            // SAFETY: checked invariant in the constructor
            unsafe {
                std::hint::unreachable_unchecked();
            }
        }
    }

    pub fn pos(&self) -> Position {
        self.assert_invariant();
        tree::root(self.heap).unwrap()
    }

    pub fn as_ref(&self) -> &S::Item {
        self.heap.get(self.pos())
    }

    pub fn as_mut(&mut self) -> &mut S::Item {
        self.assert_invariant();

        // there is not need to restore the heap if it is the only element in the heap
        if tree::child(self.heap, self.pos(), 0).is_some() {
            self.sift = true;
        }

        self.heap.get_mut(self.pos())
    }

    pub fn restore(&mut self, ord: &impl Ordering<S::Key>) -> bool {
        if self.sift {
            self.sift = false;
            self.assert_invariant();
            self.heap.sift_down(self.pos(), ord) != self.pos()
        } else {
            false
        }
    }

    pub fn ignore_mutation(&mut self) {
        self.sift = false;
    }

    pub fn heap_incoherent(&self) -> &S {
        self.assert_invariant();
        self.heap
    }

    pub fn heap_mut(&mut self) -> &mut S {
        self.assert_invariant();
        assert!(!self.sift);
        self.heap
    }
}
