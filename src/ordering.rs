//! This module defines how the heap elements should ordered relative to each other.
//! 
//! This module allos us to choose, whether we want to have large elements first (the [`max heap`]), or if we want small elements first (the [`min heap`]).
//! It also allows to use the [`Ord`] trait to compare the elements, or some custom comparison function.
//! Or if we want to have a priority queue, and compare elements only by their priority, not the keys.
//! 
//! The primary trait that encompasses all this logic is [`Ordering`].
//! This trait should not be used directly, as its methods are essentially private.
//! It is exposed only to allow its usage in trait bounds.
//! 
//! So the primary structs that implement [`Ordering`] are [`MaxHeap`] and [`MinHeap`].
//! They are constructed via one of their constructors. Like [`MaxHeap::natural`] or [`MinHeap::by_key`].
//! 
//! See [`MaxHeap`] and [`MinHeap`] for details.
//! 
//! [`max heap`]: MaxHeap
//! [`min heap`]: MinHeap

use std::cmp;

use crate::ConstDefault;

/// Private. Do not use nor implement this trait.
/// 
/// It is public only to allow its usage in trait bounds.
/// 
/// See [`module`] docs for details.
/// 
/// [`module`]: crate::ordering
pub trait Ordering<T> {
    /// Should we sift the `elt` up, above the `parent`?
    fn should_sift_up(&self, elt: &T, parent: &T) -> bool;
    /// Should we sift the `elt` down, below the `child`?
    fn should_sift_down(&self, elt: &T, child: &T) -> bool;
    
    /// Determines which of two elements should be the root when merging subtrees.
    ///
    /// Returns true if `b` should be the root
    fn select_upper(&self, a: &T, b: &T) -> bool {
        self.should_sift_up(b, a)
    }
}

/// Creates a max heap, where larger elements are prioritized.
///
/// In a max heap, the largest element is always at the top and will be returned first by `pop()`.
/// This is the default behavior of [`std::collections::BinaryHeap`].
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
/// assert_eq!(heap.pop(), Some(5)); // Largest element first
/// assert_eq!(heap.pop(), Some(3));
/// assert_eq!(heap.pop(), Some(1));
/// ```
#[derive(Default)]
pub struct MaxHeap<C = Natural>(C);

impl<T, C: Cmp<T>> Ordering<T> for MaxHeap<C> {
    fn should_sift_up(&self, elt: &T, parent: &T) -> bool {
        self.0.cmp(elt, parent).is_gt()
    }
    fn should_sift_down(&self, elt: &T, child: &T) -> bool {
        self.0.cmp(elt, child).is_lt()
    }
}

impl<C: ConstDefault> ConstDefault for MaxHeap<C> {
    const DEFAULT: Self = MaxHeap(C::DEFAULT);
}

/// Creates a min heap, where smaller elements are prioritized.
///
/// In a min heap, the smallest element is always at the top and will be returned first by `pop()`.
/// This is similar to using [`std::collections::BinaryHeap`] with [`std::cmp::Reverse`] wrapper.
///
/// # Examples
///
/// ```
/// use mheap::{VecHeap, MinHeap};
///
/// let mut heap = VecHeap::<i32, MinHeap>::new();
/// heap.push(3);
/// heap.push(1);
/// heap.push(5);
///
/// assert_eq!(heap.pop(), Some(1)); // Smallest element first
/// assert_eq!(heap.pop(), Some(3));
/// assert_eq!(heap.pop(), Some(5));
/// ```
#[derive(Default)]
pub struct MinHeap<C = Natural>(C);

impl<T, C: Cmp<T>> Ordering<T> for MinHeap<C> {
    fn should_sift_up(&self, elt: &T, parent: &T) -> bool {
        self.0.cmp(elt, parent).is_lt()
    }
    fn should_sift_down(&self, elt: &T, child: &T) -> bool {
        self.0.cmp(elt, child).is_gt()
    }
}

impl<C: ConstDefault> ConstDefault for MinHeap<C> {
    const DEFAULT: Self = MinHeap(C::DEFAULT);
}

impl MaxHeap {
    /// Creates a new `MaxHeap` that uses the default `Ord` implementation for comparison.
    ///
    /// # Examples
    ///
    /// ```
    /// use mheap::{VecHeap, MaxHeap};
    ///
    /// let mut heap = VecHeap::<i32, _>::with_ordering(MaxHeap::natural());
    /// heap.push(3);
    /// heap.push(1);
    /// heap.push(5);
    /// assert_eq!(heap.pop(), Some(5));
    /// ```
    pub fn natural() -> MaxHeap {
        MaxHeap(Natural)
    }

    /// Creates a new `MaxHeap` with a custom comparison function.
    ///
    /// The comparsion function is equivalent to the `Ord::cmp` implementation.
    ///
    /// # Examples
    ///
    /// ```
    /// use mheap::{VecHeap, MaxHeap};
    /// use std::cmp::Ordering;
    ///
    /// let mut heap = VecHeap::with_ordering(
    ///     MaxHeap::by(|a: &i32, b| a.abs().cmp(&b.abs())) // compare by absolute values
    /// );
    /// heap.push(3);
    /// heap.push(1);
    /// heap.push(-5);
    /// assert_eq!(heap.pop(), Some(-5));
    /// assert_eq!(heap.pop(), Some(3));
    /// ```
    pub fn by<T, F: Fn(&T, &T) -> cmp::Ordering>(cmp: F) -> MaxHeap<ByCmp<F>> {
        MaxHeap(ByCmp(cmp))
    }

    /// Creates a new `MaxHeap` that compares elements by a key extraction function.
    ///
    /// The key extraction function should return a value that implements `Ord`.
    /// Elements with larger keys will be prioritized.
    ///
    /// # Examples
    ///
    /// ```
    /// use mheap::{VecHeap, MaxHeap};
    ///
    /// let mut heap = VecHeap::with_ordering(
    ///     MaxHeap::by_key(|item: &(&str, i32)| item.1) // Compare by the second field
    /// );
    /// heap.push(("low", 1));
    /// heap.push(("high", 10));
    /// heap.push(("medium", 5));
    ///
    /// assert_eq!(heap.pop(), Some(("high", 10)));
    /// assert_eq!(heap.pop(), Some(("medium", 5)));
    /// assert_eq!(heap.pop(), Some(("low", 1)));
    /// ```
    pub fn by_key<T, K: Ord, F: Fn(&T) -> K>(key: F) -> MaxHeap<ByKey<F>> {
        MaxHeap(ByKey(key))
    }
}

impl MinHeap {
    /// Creates a new `MinHeap` that uses the default `Ord` implementation for comparison.
    ///
    /// # Examples
    ///
    /// ```
    /// # use mheap::{VecHeap, MinHeap};
    ///
    /// let mut heap = VecHeap::<i32, _>::with_ordering(MinHeap::natural());
    /// heap.push(3);
    /// heap.push(1);
    /// heap.push(5);
    /// assert_eq!(heap.pop(), Some(1));
    /// ```
    pub fn natural() -> MinHeap {
        MinHeap(Natural)
    }

    /// Creates a new `MinHeap` with a custom comparison function.
    ///
    /// The comparsion function is equivalent to the `Ord::cmp` implementation.
    ///
    /// # Examples
    ///
    /// ```
    /// # use mheap::{VecHeap, MinHeap};
    /// # use std::cmp::Ordering;
    ///
    /// let mut heap = VecHeap::with_ordering(
    ///     MinHeap::by(|a: &i32, b| a.abs().cmp(&b.abs())) // compare by absolute values
    /// );
    /// heap.push(-3);
    /// heap.push(1);
    /// heap.push(5);
    /// assert_eq!(heap.pop(), Some(1));
    /// assert_eq!(heap.pop(), Some(-3));
    /// ```
    pub fn by<T, F: Fn(&T, &T) -> cmp::Ordering>(cmp: F) -> MinHeap<ByCmp<F>> {
        MinHeap(ByCmp(cmp))
    }

    /// Creates a new `MinHeap` that compares elements by a key extraction function.
    ///
    /// The key extraction function should return a value that implements `Ord`.
    /// Elements with larger keys will be prioritized.
    ///
    /// # Examples
    ///
    /// ```
    /// use mheap::{VecHeap, MinHeap};
    ///
    /// let mut heap = VecHeap::with_ordering(
    ///     MinHeap::by_key(|item: &(&str, i32)| item.1) // Compare by the second field
    /// );
    /// heap.push(("low", 1));
    /// heap.push(("high", 10));
    /// heap.push(("medium", 5));
    ///
    /// assert_eq!(heap.pop(), Some(("low", 1)));
    /// assert_eq!(heap.pop(), Some(("medium", 5)));
    /// assert_eq!(heap.pop(), Some(("high", 10)));
    /// ```
    pub fn by_key<T, K: Ord, F: Fn(&T) -> K>(key: F) -> MinHeap<ByKey<F>> {
        MinHeap(ByKey(key))
    }
}

/// Private. Do not use nor implement this trait.
/// 
/// It is public only to allow its usage in trait boudns
pub trait Cmp<T> {
    fn cmp(&self, a: &T, b: &T) -> cmp::Ordering;
}

/// Natural ordering using the default `Ord` implementation.
///
/// This is the default comparison strategy that uses the element's
/// natural ordering as defined by its `Ord` implementation.
#[derive(Default, Clone, Copy)]
pub struct Natural;

impl<T: Ord> Cmp<T> for Natural {
    fn cmp(&self, a: &T, b: &T) -> cmp::Ordering {
        a.cmp(b)
    }
}

impl ConstDefault for Natural {
    const DEFAULT: Self = Natural;
}

/// A comparison implementation that uses a custom function.
///
/// It's useful when you need custom comparison logic that doesn't fit
/// the key extraction pattern (see [`ByKey`]).
pub struct ByCmp<F>(F);

impl<T, F: Fn(&T, &T) -> cmp::Ordering> Cmp<T> for ByCmp<F> {
    fn cmp(&self, a: &T, b: &T) -> cmp::Ordering {
        self.0(a, b)
    }
}

/// A comparison implementation that compares elements by an extracted key.
///
/// Use it via [`MaxHeap::by_key`] and [`MinHeap::by_key`]
pub struct ByKey<F>(F);

impl<T, F: Fn(&T) -> K, K: Ord> Cmp<T> for ByKey<F> {
    fn cmp(&self, a: &T, b: &T) -> cmp::Ordering {
        self.0(a).cmp(&self.0(b))
    }
}
