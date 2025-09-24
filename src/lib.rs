#![deny(unsafe_op_in_unsafe_fn)]

//! This crate provides classical binary heaps.
//! Like [`std::collections::BinaryHeap`], but with more flexibility oriented design.
//!
//! General api is similar to [`std::collections::BinaryHeap`]:
//! ```
//! # use mheap::{VecHeap, MaxHeap};
//!
//! let mut heap = VecHeap::<_, MaxHeap>::new();
//! heap.push(3);
//! heap.push(15);
//! heap.push(1);
//!
//! let mut data = Vec::new();
//! while let Some(x) = heap.pop() {
//!     data.push(x);
//! }
//! assert_eq!(data, vec![15, 3, 1]);
//! ```
//!
//! To use the crate, you need to select a few options:
//!
//! First you select the heap `storage`.
//! It represents how the heap is stored in memory and what additional operations are needed.
//! Currently there are two storages:
//! * [`VecHeap`] - stores elements in a plain [`Vec`] and nothing else. Analogous to [`std::collections::BinaryHeap`].
//! * [`IndexableHeap`] - similar to [`VecHeap`], but allows to access elements by an opaque [`Idx`]
//!
//! Then you select how the elements should be sorted - an [`Ordering`].
//! Two primary orderings are:
//! * [`MaxHeap`] - puts largest element on top of the heap. Like the [`std::collections::BinaryHeap`].
//! * [`MinHeap`] - puts smallest element on top of the heap. Like the [`std::collections::BinaryHeap`] with [`Reverse`] wrapper.
//!
//! You can also compare elements by ad-hoc orderings. For example:
//! ```
//! # use mheap::{VecHeap, MaxHeap};
//!
//! let mut heap = VecHeap::with_ordering(MaxHeap::by_key(|it: &(_, _)| it.0));
//! heap.push((3, 1));  
//! heap.push((15, 2));
//! heap.push((1, 3));
//!
//! let mut data = Vec::new();
//! while let Some(x) = heap.pop() {
//!     data.push(x);
//! }
//! assert_eq!(data, vec![(15, 2), (3, 1), (1, 3)]);
//! ```
//!
//! See [`MaxHeap`] and [`MinHeap`] for details.
//!
//! [`Idx`]: indexable_heap::Idx
//! [`Reverse`]: std::cmp::Reverse
//! [`Ordering`]: crate::ordering::Ordering

mod hole;
pub mod ordering;
mod sift;
mod storage;
mod tree;

mod raw_heap;

pub mod indexable_heap;
mod indexable_vec;
pub mod vec_heap;

pub(crate) use raw_heap::RawHeap;

pub use crate::{
    indexable_heap::IndexableHeap,
    ordering::{MaxHeap, MinHeap},
    vec_heap::VecHeap,
};

pub type Position = usize;

/// A hack to have [`Default`] trait in const contexts. Used for [`Ordering`] impls.
/// 
/// [`Ordering`]: crate::ordering::Ordering
pub trait ConstDefault: Default {
    const DEFAULT: Self;
}
