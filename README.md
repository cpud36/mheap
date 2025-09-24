
A flexible binary heap implementation for Rust with more design flexibility than the standard library's `BinaryHeap`.

# Overview

This crate provides classical binary heaps similar to [`std::collections::BinaryHeap`], but with a more flexible, modular design that allows you to choose both the storage mechanism and the ordering behavior.

# Quick Start

```rust
use mheap::{VecHeap, MaxHeap};

let mut heap = VecHeap::<_, MaxHeap>::new();
heap.push(3);
heap.push(15);
heap.push(1);

let mut data = Vec::new();
while let Some(x) = heap.pop() {
    data.push(x);
}
assert_eq!(data, vec![15, 3, 1]);
```

# Usage

## Choose Storage

Select how the heap is stored in memory:

- **`VecHeap`** - Stores elements in a plain `Vec`, analogous to `std::collections::BinaryHeap`
- **`IndexableHeap`** - Similar to `VecHeap`, but allows accessing elements by an opaque `Idx`

## Choose Ordering

Select how elements should be sorted:

- **`MaxHeap`** - Puts the largest element on top (like `std::collections::BinaryHeap`)
- **`MinHeap`** - Puts the smallest element on top (like `std::collections::BinaryHeap` with `Reverse` wrapper)

## 3. Custom Orderings

You can compare elements using custom orderings:

```rust
use mheap::{VecHeap, MaxHeap};

let mut heap = VecHeap::with_ordering(MaxHeap::by_key(|it: &(_, _)| it.0));
heap.push((3, 1));  
heap.push((15, 2));
heap.push((1, 3));

let mut data = Vec::new();
while let Some(x) = heap.pop() {
    data.push(x);
}
assert_eq!(data, vec![(15, 2), (3, 1), (1, 3)]);
```

# Examples

## Basic Max Heap

```rust
use mheap::{VecHeap, MaxHeap};

let mut heap = VecHeap::<i32, MaxHeap>::new();
heap.push(42);
heap.push(1);
heap.push(100);

assert_eq!(heap.pop(), Some(100));
assert_eq!(heap.pop(), Some(42));
assert_eq!(heap.pop(), Some(1));
assert_eq!(heap.pop(), None);
```

## Min Heap

```rust
use mheap::{VecHeap, MinHeap};

let mut heap = VecHeap::<i32, MinHeap>::new();
heap.push(42);
heap.push(1);
heap.push(100);

assert_eq!(heap.pop(), Some(1));
assert_eq!(heap.pop(), Some(42));
assert_eq!(heap.pop(), Some(100));
```

## Indexable Heap

```rust
use mheap::{IndexableHeap, MaxHeap};

let mut heap = IndexableHeap::<i32, MaxHeap>::new();
let idx1 = heap.push(42);
let idx2 = heap.push(1);
let idx3 = heap.push(100);

// Access elements by index
assert_eq!(heap.by_index(idx1), Some(&42));
assert_eq!(heap.by_index(idx2), Some(&1));
assert_eq!(heap.by_index(idx3), Some(&100));

// Update elements
*heap.by_index_mut(idx1) = 150;
assert_eq!(heap.peek(), Some(&150));
```

