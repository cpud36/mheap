//! A heap that tracks where elements move and allows access by index.
//! 
//! See [`IndexableHeap`] for details.

use std::ops::{Deref, DerefMut};

use crate::{
    ConstDefault, ordering::Ordering, Position, RawHeap, raw_heap,
    indexable_vec::IndexableVec,
};

pub use crate::indexable_vec::Idx;

/// A heap that tracks where elements move and allows access by index.
///
/// Unlike [`VecHeap`], this heap maintains a mapping from opaque indices to
/// heap positions, allowing you to access and modify elements by their index
/// even after the heap has been reordered.
///
/// Use the `O` generic parameter to select [`MaxHeap`] or [`MinHeap`].
///
/// It stores elements in a [`Vec`] like [`VecHeap`] but also tracks their positions in a side map.
/// On push it returns an opaque handle [`Idx`] to the element.
/// You can later use it to get (& or &mut) access to the element.
/// See [`IndexableHeap::by_index_mut`] for details.
///
/// # Examples
///
/// ```
/// use mheap::{IndexableHeap, MinHeap};
///
/// let mut heap = IndexableHeap::<i32, MinHeap>::new();
/// heap.push(16);
/// let idx = heap.push(7);
/// heap.push(5);
///
/// assert_eq!(heap.peek(), Some(&5));
/// *heap.by_index_mut(idx) = 2; // Modify element by index
/// assert_eq!(heap.pop(), Some(2)); // Modified element is now at top
/// ```
///
/// # Time complexity
///
/// | Operation | Time complexity |
/// |-----------|-----------------|
/// | `push`    | *O*(1)~         |
/// | `pop`     | *O*(log(*n*))   |
/// | `peek`    | *O*(1)          |
///
/// The `push` operation has expected *O*(1) complexity.
/// See its documentation for details.
///
/// [`VecHeap`]: crate::VecHeap
/// [`MaxHeap`]: crate::MaxHeap
/// [`MinHeap`]: crate::MinHeap
pub struct IndexableHeap<T, O> {
    data: IndexableVec<T>,
    ord: O,
}

impl<T, O> IndexableHeap<T, O> {
    /// Creates a new empty heap.
    pub const fn new() -> Self
    where
        O: ConstDefault,
    {
        Self {
            data: IndexableVec::new(),
            ord: O::DEFAULT,
        }
    }

    /// Creates a new empty heap with the specified capacity.
    ///
    /// The heap will be able to hold at least `capacity` elements without reallocating.
    pub fn with_capacity(capacity: usize) -> Self
    where
        O: Default,
    {
        Self {
            data: IndexableVec::with_capacity(capacity),
            ord: O::default(),
        }
    }

    /// Creates a new empty heap with the specified ordering.
    pub const fn with_ordering(ord: O) -> Self {
        Self {
            data: IndexableVec::new(),
            ord,
        }
    }

    /// Creates a new empty heap with the specified capacity and ordering.
    ///
    /// The heap will be able to hold at least `capacity` elements without reallocating.
    pub fn with_capacity_and_ordering(capacity: usize, ord: O) -> Self {
        Self {
            data: IndexableVec::with_capacity(capacity),
            ord,
        }
    }

    /// Returns the number of elements in the heap.
    ///
    /// # Examples
    ///
    /// ```
    /// # use mheap::{IndexableHeap, MaxHeap};
    ///
    /// let mut heap = IndexableHeap::<i32, MaxHeap>::new();
    /// assert_eq!(heap.len(), 0);
    /// 
    /// heap.push(1);
    /// assert_eq!(heap.len(), 1);
    /// ```
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Returns the capacity of the heap.
    ///
    /// # Examples
    ///
    /// ```
    /// # use mheap::{IndexableHeap, MaxHeap};
    /// ```
    pub fn capacity(&self) -> usize {
        self.data.capacity()
    }

    /// Returns `true` if the heap is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// # use mheap::{IndexableHeap, MaxHeap};
    ///
    /// let mut heap = IndexableHeap::<i32, MaxHeap>::new();
    /// assert!(heap.is_empty());
    /// 
    /// heap.push(1);
    /// assert!(!heap.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

impl<T, O: Ordering<T>> IndexableHeap<T, O> {
    /// Returns a reference to the top element in the heap, or `None` if it is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// # use mheap::{IndexableHeap, MaxHeap};
    ///
    /// let mut heap = IndexableHeap::<i32, MaxHeap>::new();
    /// assert_eq!(heap.peek(), None);
    ///
    /// heap.push(3);
    /// heap.push(1);
    /// heap.push(5);
    /// assert_eq!(heap.peek(), Some(&5));
    /// assert_eq!(heap.peek(), Some(&5)); // Still the same
    /// ```
    ///
    /// # Time complexity
    ///
    /// *O*(1)
    pub fn peek(&self) -> Option<&T> {
        self.data.peek()
    }

    /// Returns a mutable reference to the top element in the heap, or `None` if it is empty.
    ///
    /// Like [`IndexableHeap::by_index_mut`], this method allows you to change element ordering relative to other elements.
    /// It will safely update the heap when the wrapper is dropped.
    ///
    /// # Examples
    ///
    /// ```
    /// # use mheap::{IndexableHeap, MaxHeap};
    ///
    /// let mut heap = IndexableHeap::<i32, MaxHeap>::new();
    /// heap.push(3);
    /// heap.push(1);
    /// heap.push(5);
    ///
    /// if let Some(mut val) = heap.peek_mut() {
    ///     assert_eq!(*val, 5);
    ///     *val = 0; // Change the top element
    /// }
    /// assert_eq!(heap.peek(), Some(&3)); // Heap is automatically reordered
    /// ```
    ///
    /// # Time complexity
    ///
    /// If the item is modified then the worst case time complexity is *O*(log(*n*)),
    /// otherwise it's *O*(1).
    pub fn peek_mut(&mut self) -> Option<PeekMut<'_, T, O>> {
        RawHeap::peek_mut(&mut self.data).map(|raw| PeekMut {
            raw,
            ord: &self.ord,
        })
    }

    /// Get a reference to an element by its index (obtained via [`IndexableHeap::push`]).
    ///
    /// # Examples
    ///
    /// ```
    /// # use mheap::{IndexableHeap, MaxHeap};
    ///
    /// let mut heap = IndexableHeap::<i32, MaxHeap>::new();
    /// let idx = heap.push(3);
    /// heap.push(1);
    /// heap.push(5);
    ///
    /// assert_eq!(heap.by_index(idx), &3);
    /// ```
    /// 
    /// # Panics
    /// 
    /// If the index is invalid, the method might, or might not panic.
    pub fn by_index(&self, index: Idx<T>) -> &T {
        let pos = self.data.index_to_pos(index);
        self.data.get(pos)
    }

    /// Get a mutable reference to an element by its index.
    ///
    /// This method returns a wrapper that will restore the heap invariant when dropped.
    /// If the element ordering relative to other elements has changed, it will be put into its new position (when the [`GetMut`] wrapper is dropped).
    ///
    /// # Examples
    ///
    /// ```
    /// use mheap::{IndexableHeap, MinHeap};
    ///
    /// let mut heap = IndexableHeap::<i32, MinHeap>::new();
    /// heap.push(16);
    /// let idx = heap.push(7);
    /// heap.push(5);
    ///
    /// assert_eq!(heap.peek(), Some(&5));
    /// *heap.by_index_mut(idx) = 2;
    /// assert_eq!(heap.pop(), Some(2));
    ///
    /// assert_eq!(heap.pop(), Some(5));
    /// assert_eq!(heap.pop(), Some(16));
    /// assert_eq!(heap.pop(), None);
    /// ```
    ///
    /// # Panics
    ///
    /// If the element was removed from the heap.
    ///
    /// # Time complexity
    ///
    /// If the item is modified then the worst case time complexity is *O*(log(*n*)),
    /// otherwise it's *O*(1).
    pub fn by_index_mut(&mut self, index: Idx<T>) -> GetMut<'_, T, O> {
        let pos = self.data.index_to_pos(index);
        GetMut::new(self, pos)
    }

    /// Pushes an item onto the heap and returns an index to it.
    ///
    /// The returned index can be used later to access the element even after
    /// the heap has been reordered by other operations.
    ///
    /// # Examples
    ///
    /// ```
    /// use mheap::{IndexableHeap, MaxHeap};
    ///
    /// let mut heap = IndexableHeap::<i32, MaxHeap>::new();
    /// let idx = heap.push(3);
    /// heap.push(1);
    /// heap.push(5);
    /// assert_eq!(heap.len(), 3);
    /// 
    /// // Later, we can still access the element by its index
    /// *heap.by_index_mut(idx) = 10;
    /// ```
    ///
    /// # Time complexity
    ///
    /// The expected cost of `push`, averaged over every possible ordering of
    /// the elements being pushed, and over a sufficiently large number of
    /// pushes, is *O*(1). This is the most meaningful cost metric when pushing
    /// elements that are *not* already in any sorted pattern.
    ///
    /// The time complexity degrades if elements are pushed in predominantly
    /// ascending order(for MaxHeap). In the worst case, elements are pushed in ascending
    /// sorted order and the amortized cost per push is *O*(log(*n*)) against a heap
    /// containing *n* elements.
    /// 
    /// The worst case cost of a *single* call to `push` is *O*(*n*). The worst case
    /// occurs when capacity is exhausted and needs a resize. The resize cost
    /// has been amortized in the previous figures.
    pub fn push(&mut self, item: T) -> Idx<T> {
        let pos = self.data.len();
        let index = self.data.push(item);
        self.data.sift_up(pos, &self.ord);
        index
    }

    /// Removes the top element from the heap and returns it, or `None` if it is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use mheap::{IndexableHeap, MaxHeap};
    ///
    /// let mut heap = IndexableHeap::<i32, MaxHeap>::new();
    /// heap.push(3);
    /// heap.push(1);
    /// heap.push(5);
    ///
    /// assert_eq!(heap.pop(), Some(5));
    /// assert_eq!(heap.pop(), Some(3));
    /// assert_eq!(heap.pop(), Some(1));
    /// assert_eq!(heap.pop(), None);
    /// ```
    ///
    /// # Time complexity
    ///
    /// The worst case cost of `pop` on a heap containing *n* elements is *O*(log(*n*)).
    pub fn pop(&mut self) -> Option<T> {
        let item = self.data.pop()?;
        Some(self.data.pop_swap(item, &self.ord))
    }

    /// Reserves capacity for at least `additional` elements more than the
    /// current length. The allocator may reserve more space to speculatively
    /// avoid frequent allocations. After calling `reserve`,
    /// capacity will be greater than or equal to `self.len() + additional`.
    /// Does nothing if capacity is already sufficient.
    /// 
    /// # Panics
    ///
    /// Panics if the new capacity overflows [`usize`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use mheap::{IndexableHeap, MaxHeap};
    ///
    /// let mut heap = IndexableHeap::<i32, MaxHeap>::new();
    /// heap.reserve(100);
    /// assert!(heap.capacity() >= 100);
    /// ```
    pub fn reserve(&mut self, additional: usize) {
        self.data.reserve(additional);
    }

    /// Reserves the minimum capacity for at least `additional` elements more than
    /// the current length. Unlike [`reserve`], this will not
    /// deliberately over-allocate to speculatively avoid frequent allocations.
    /// After calling `reserve_exact`, capacity will be greater than or equal to
    /// `self.len() + additional`. Does nothing if the capacity is already
    /// sufficient.
    ///
    /// # Panics
    ///
    /// Panics if the new capacity overflows [`usize`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use mheap::{IndexableHeap, MaxHeap};
    ///
    /// let mut heap = IndexableHeap::<i32, MaxHeap>::new();
    /// heap.reserve_exact(100);
    /// assert!(heap.capacity() >= 100);
    /// ```
    ///
    /// [`reserve`]: IndexableHeap::reserve
    pub fn reserve_exact(&mut self, additional: usize) {
        self.data.reserve_exact(additional);
    }

    /// Discards as much additional capacity as possible.
    ///
    /// # Examples
    ///
    /// ```
    /// use mheap::{IndexableHeap, MaxHeap};
    ///
    /// let mut heap: IndexableHeap<i32, MaxHeap> = IndexableHeap::with_capacity(100);
    /// assert!(heap.capacity() >= 100);
    /// heap.push(4);
    /// heap.shrink_to_fit();
    /// assert!(heap.capacity() == 1);
    /// ```
    pub fn shrink_to_fit(&mut self) {
        self.data.shrink_to_fit();
    }

    /// Discards capacity with a lower bound.
    ///
    /// The capacity will remain at least as large as both the length
    /// and the supplied value.
    ///
    /// If the current capacity is less than the lower limit, this is a no-op.
    ///
    /// # Examples
    ///
    /// ```
    /// use mheap::{IndexableHeap, MaxHeap};
    ///
    /// let mut heap: IndexableHeap<i32, MaxHeap> = IndexableHeap::with_capacity(100);
    /// assert!(heap.capacity() >= 100);
    /// heap.shrink_to(10);
    /// assert!(heap.capacity() >= 10);
    /// ```
    pub fn shrink_to(&mut self, min_capacity: usize) {
        self.data.shrink_to(min_capacity);
    }
}

/// Structure wrapping a mutable reference to the top item on an [`IndexableHeap`].
///
/// This `struct` is created by the [`peek_mut`] method on [`IndexableHeap`]. See
/// its documentation for more.
///
/// [`peek_mut`]: IndexableHeap::peek_mut
pub struct PeekMut<'a, T, O: Ordering<T>> {
    raw: raw_heap::PeekMut<'a, IndexableVec<T>>,
    ord: &'a O,
}

impl<'a, T, O: Ordering<T>> Drop for PeekMut<'a, T, O> {
    fn drop(&mut self) {
        self.restore();
    }
}

impl<'a, T, O: Ordering<T>> Deref for PeekMut<'a, T, O> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.raw.as_ref()
    }
}

impl<'a, T, O: Ordering<T>> DerefMut for PeekMut<'a, T, O> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.raw.as_mut()
    }
}

impl<'a, T, O: Ordering<T>> PeekMut<'a, T, O> {
    fn restore(&mut self) {
        self.raw.restore(self.ord);
    }

    /// Returns the index of the peeked element.
    ///
    /// This index can be used later to access the same element even after
    /// the heap has been reordered.
    ///
    /// # Examples
    ///
    /// ```
    /// use mheap::{IndexableHeap, MinHeap};
    ///
    /// let mut heap = IndexableHeap::<i32, MinHeap>::new();
    /// heap.push(3);
    /// heap.push(7);
    /// heap.push(5);
    ///
    /// let peek = heap.peek_mut().unwrap();
    /// assert_eq!(*peek, 3);
    /// let idx = peek.index();
    /// drop(peek);
    /// 
    /// heap.push(1);
    /// assert_eq!(heap.peek(), Some(&1));
    /// 
    /// let entry = heap.by_index_mut(idx);
    /// assert_eq!(*entry, 3);
    /// ```
    pub fn index(&self) -> Idx<T> {
        self.raw.heap_incoherent().pos_to_index(self.raw.pos())
    }

    /// Removes the peeked value from the heap and returns it.
    ///
    /// This method consumes the `PeekMut` and removes the top element from the heap.
    /// The heap invariant is maintained automatically.
    ///
    /// # Examples
    ///
    /// ```
    /// # use mheap::{IndexableHeap, MaxHeap};
    ///
    /// let mut heap = IndexableHeap::<i32, MaxHeap>::new();
    /// heap.push(3);
    /// heap.push(1);
    /// heap.push(5);
    ///
    /// let peek = heap.peek_mut().unwrap();
    /// let value = peek.pop();
    /// assert_eq!(value, 5);
    /// assert_eq!(heap.len(), 2);
    /// assert_eq!(heap.peek(), Some(&3));
    /// ```
    ///
    /// # Time complexity
    ///
    /// The worst case cost of `pop` on a heap containing *n* elements is *O*(log(*n*)).
    pub fn pop(mut self) -> T {
        // We don't care if the element was mutated, as we will remove in the next line
        self.raw.ignore_mutation();

        let heap = self.raw.heap_mut();
        let item = heap.pop().unwrap();
        heap.pop_swap(item, self.ord)
    }
}

/// Structure wrapping a mutable reference to an element in an [`IndexableHeap`].
///
/// This `struct` is created by the [`by_index_mut`] method on [`IndexableHeap`]. See
/// its documentation for more.
///
/// [`by_index_mut`]: IndexableHeap::by_index_mut
pub struct GetMut<'a, T, O: Ordering<T>> {
    heap: &'a mut IndexableHeap<T, O>,
    pos: Position,
    sift: bool,
}

impl<'a, T, O: Ordering<T>> Deref for GetMut<'a, T, O> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<'a, T, O: Ordering<T>> DerefMut for GetMut<'a, T, O> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut()
    }
}

impl<'a, T, O: Ordering<T>> Drop for GetMut<'a, T, O> {
    fn drop(&mut self) {
        self.restore();
    }
}

impl<'a, T, O: Ordering<T>> GetMut<'a, T, O> {
    fn new(heap: &'a mut IndexableHeap<T, O>, pos: Position) -> Self {
        assert!(pos < heap.data.len());
        Self {
            heap,
            pos,
            sift: false,
        }
    }

    fn pos(&self) -> Position {
        if self.pos >= self.heap.data.len() {
            // SAFETY: checked invariant in the new call
            unsafe {
                std::hint::unreachable_unchecked();
            }
        }
        self.pos
    }

    /// Returns the index of the element.
    ///
    /// This index can be used later to access the same element even after
    /// the heap has been reordered.
    ///
    /// # Examples
    ///
    /// ```
    /// use mheap::{IndexableHeap, MaxHeap};
    ///
    /// let mut heap = IndexableHeap::<i32, MaxHeap>::new();
    /// let idx = heap.push(5);
    /// heap.push(7);
    /// heap.push(3);
    ///
    /// let entry = heap.by_index_mut(idx);
    /// assert_eq!(*entry, 5);
    /// assert_eq!(entry.index(), idx);
    /// ```
    pub fn index(&self) -> Idx<T> {
        self.heap.data.pos_to_index(self.pos())
    }

    fn as_ref(&self) -> &T {
        self.heap.data.get(self.pos())
    }

    fn as_mut(&mut self) -> &mut T {
        self.sift = true;

        self.heap.data.get_mut(self.pos())
    }

    fn restore(&mut self) -> bool {
        if self.sift {
            self.sift = false;
            self.heap.data.fixup_sift(self.pos(), &self.heap.ord) != self.pos()
        } else {
            false
        }
    }

    /// Removes the element from the heap and returns it.
    ///
    /// This method consumes the `GetMut` and removes the element from the heap.
    /// The heap invariant is maintained automatically.
    ///
    /// # Examples
    ///
    /// ```
    /// # use mheap::{IndexableHeap, MinHeap};
    /// 
    /// let mut heap = IndexableHeap::<i32, MinHeap>::new();
    /// heap.push(16);
    /// let idx = heap.push(7);
    /// heap.push(5);
    ///
    /// let entry = heap.by_index_mut(idx);
    /// assert_eq!(*entry, 7);
    /// let item = entry.remove();
    /// assert_eq!(item, 7);
    ///
    /// assert_eq!(heap.pop(), Some(5));
    /// assert_eq!(heap.pop(), Some(16));
    /// assert_eq!(heap.pop(), None);
    /// ```
    ///
    /// # Time complexity
    ///
    /// Worst case is *O*(log(*n*))
    pub fn remove(mut self) -> T {
        // We don't care if the element was mutated, as we will remove it in the next line
        self.sift = false;

        let pos = self.pos();
        let item = self.heap.data.swap_remove(pos);
        // In case it was the last element, we don't need to fix its position
        if pos < self.heap.data.len() {
            self.heap.data.fixup_sift_to_bottom(pos, &self.heap.ord);
        }
        item
    }
}
