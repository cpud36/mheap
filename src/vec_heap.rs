//! A simple heap stored in a [`Vec`]. Analogous to [`std::collections::BinaryHeap`].
//! 
//! See [`VecHeap`] for details.

use std::ops::{Deref, DerefMut};

use crate::{ConstDefault, ordering::Ordering, RawHeap, raw_heap};

/// A simple heap stored in a [`Vec`]. Analogous to [`std::collections::BinaryHeap`].
///
/// Use the `O` generic parameter to select [`MaxHeap`] or [`MinHeap`].
/// See [`crate::ordering`] for details.
///
/// # Examples
///
/// ```
/// # use mheap::{VecHeap, MaxHeap, MinHeap};
///
/// let mut max_heap = VecHeap::<i32, MaxHeap>::new();
/// max_heap.push(3);
/// max_heap.push(1);
/// max_heap.push(5);
/// assert_eq!(max_heap.pop(), Some(5));
/// ```
///
/// # Time complexity
///
/// | Operation | Time complexity |
/// |-----------|----------------|
/// | `push`    | *O*(1)~        |
/// | `pop`     | *O*(log(*n*))  |
/// | `peek`    | *O*(1)         |
///
/// The value of `push` is an expected complexity.
///
/// [`MaxHeap`]: crate::MaxHeap
/// [`MinHeap`]: crate::MinHeap
pub struct VecHeap<T, O> {
    data: Vec<T>,
    ord: O,
}

impl<T, O> VecHeap<T, O> {
    /// Creates a new empty heap
    pub const fn new() -> Self
    where
        O: ConstDefault,
    {
        Self {
            data: Vec::new(),
            ord: O::DEFAULT,
        }
    }

    /// Creates a new empty heap with the specified capacity
    ///
    /// The heap will be able to hold at least `capacity` elements without reallocating.
    pub fn with_capacity(capacity: usize) -> Self
    where
        O: Default,
    {
        Self {
            data: Vec::with_capacity(capacity),
            ord: O::default(),
        }
    }

    /// Creates a new empty heap with the specified ordering.
    pub const fn with_ordering(ord: O) -> Self {
        Self {
            data: Vec::new(),
            ord,
        }
    }

    /// Creates a new empty heap with the specified capacity and ordering.
    ///
    /// The heap will be able to hold at least `capacity` elements without reallocating.
    pub fn with_capacity_and_ordering(capacity: usize, ord: O) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
            ord,
        }
    }

    /// Returns the number of elements in the heap.
    ///
    /// # Examples
    ///
    /// ```
    /// use mheap::{VecHeap, MaxHeap};
    ///
    /// let mut heap = VecHeap::<i32, MaxHeap>::new();
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
    /// use mheap::{VecHeap, MaxHeap};
    ///
    /// let mut heap = VecHeap::<i32, MaxHeap>::new();
    /// assert_eq!(heap.capacity(), 0);
    /// ```
    pub fn capacity(&self) -> usize {
        self.data.capacity()
    }

    /// Returns `true` if the heap is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use mheap::{VecHeap, MaxHeap};
    ///
    /// let mut heap = VecHeap::<i32, MaxHeap>::new();
    /// assert!(heap.is_empty());
    /// 
    /// heap.push(1);
    /// assert!(!heap.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

impl<T, O: Ordering<T>> VecHeap<T, O> {
    /// Returns a reference to the top element in the heap, or `None` if it is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use mheap::{VecHeap, MaxHeap};
    ///
    /// let mut heap = VecHeap::<i32, MaxHeap>::new();
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
    /// This method allows you to change element ordering relative to other elements.
    /// It will safely update the heap when the [`PeekMut`] wrapper is dropped.
    ///
    /// # Examples
    ///
    /// ```
    /// # use mheap::{VecHeap, MaxHeap};
    ///
    /// let mut heap = VecHeap::<i32, MaxHeap>::new();
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

    /// Pushes an item onto the heap.
    ///
    /// # Examples
    ///
    /// ```
    /// # use mheap::{VecHeap, MaxHeap};
    ///
    /// let mut heap = VecHeap::<i32, MaxHeap>::new();
    /// heap.push(3);
    /// heap.push(1);
    /// heap.push(5);
    /// assert_eq!(heap.len(), 3);
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
    pub fn push(&mut self, item: T) {
        let pos = self.data.len();
        self.data.push(item);
        self.data.sift_up(pos, &self.ord);
    }

    /// Removes the top element from the heap and returns it, or `None` if it is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use mheap::{VecHeap, MaxHeap};
    ///
    /// let mut heap = VecHeap::<i32, MaxHeap>::new();
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
    /// use mheap::{VecHeap, MaxHeap};
    ///
    /// let mut heap = VecHeap::<i32, MaxHeap>::new();
    /// heap.reserve(100);
    /// assert!(heap.capacity() >= 100);
    /// heap.push(4);
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
    /// use mheap::{VecHeap, MaxHeap};
    ///
    /// let mut heap = VecHeap::<i32, MaxHeap>::new();
    /// heap.reserve_exact(100);
    /// assert!(heap.capacity() >= 100);
    /// heap.push(4);
    /// ```
    ///
    /// [`reserve`]: VecHeap::reserve
    pub fn reserve_exact(&mut self, additional: usize) {
        self.data.reserve_exact(additional);
    }

    /// Discards as much additional capacity as possible.
    ///
    /// # Examples
    ///
    /// ```
    /// use mheap::{VecHeap, MaxHeap};
    ///
    /// let mut heap: VecHeap<i32, MaxHeap> = VecHeap::with_capacity(100);
    /// heap.push(4);
    /// assert!(heap.capacity() >= 100);
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
    /// use mheap::{VecHeap, MaxHeap};
    ///
    /// let mut heap: VecHeap<i32, MaxHeap> = VecHeap::with_capacity(100);
    /// assert!(heap.capacity() >= 100);
    /// heap.shrink_to(10);
    /// assert!(heap.capacity() >= 10);
    /// ```
    pub fn shrink_to(&mut self, min_capacity: usize) {
        self.data.shrink_to(min_capacity);
    }

    /// Moves all the elements of `other` into `self`, leaving `other` empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use mheap::{VecHeap, MaxHeap};
    ///
    /// let mut a = VecHeap::<i32, MaxHeap>::new();
    /// a.push(1);
    /// a.push(2);
    /// a.push(3);
    ///
    /// let mut b = VecHeap::<i32, MaxHeap>::new();
    /// b.push(4);
    /// b.push(5);
    ///
    /// a.append(&mut b);
    /// assert_eq!(a.len(), 5);
    /// assert!(b.is_empty());
    /// ```
    pub fn append(&mut self, other: &mut Self) {
        if self.len() < other.len() {
            std::mem::swap(self, other);
        }

        let start = self.len();
        self.data.append(&mut other.data);
        self.data.rebuild_tail(start, &self.ord);
    }
}

/// Structure wrapping a mutable reference to the top item on a [`VecHeap`].
///
/// This `struct` is created by the [`peek_mut`] method on [`VecHeap`]. See
/// its documentation for more.
///
/// [`peek_mut`]: VecHeap::peek_mut
pub struct PeekMut<'a, T, O: Ordering<T>> {
    raw: raw_heap::PeekMut<'a, Vec<T>>,
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

    /// Removes the peeked value from the heap and returns it.
    ///
    /// This method consumes the `PeekMut` and removes the top element from the heap.
    /// The heap invariant is maintained automatically.
    ///
    /// # Examples
    ///
    /// ```
    /// use mheap::{VecHeap, MaxHeap};
    ///
    /// let mut heap = VecHeap::<i32, MaxHeap>::new();
    /// heap.push(3);
    /// heap.push(1);
    /// heap.push(5);
    ///
    /// let peek = heap.peek_mut().unwrap();
    /// let value = peek.pop();
    /// assert_eq!(value, 5);
    /// assert_eq!(heap.len(), 2);
    /// ```
    ///
    /// # Time complexity
    ///
    /// *O*(log(*n*))
    pub fn pop(mut self) -> T {
        // We don't care if the element was mutated, as we will remove in the next line
        self.raw.ignore_mutation();

        let heap = self.raw.heap_mut();
        let item = heap.pop().unwrap();
        heap.pop_swap(item, self.ord)
    }
}
