use mheap::{MaxHeap, MinHeap, IndexableHeap, VecHeap};

#[test]
fn min_heap() {
    let mut heap = VecHeap::<i32, MinHeap>::new();
    heap.push(3);
    heap.push(15);
    heap.push(1);
    heap.push(42);
    heap.push(7);
    heap.push(6);
    heap.push(5);
    heap.push(64);

    let mut data = Vec::new();
    while let Some(x) = heap.pop() {
        data.push(x);
    }
    assert_eq!(data, vec![1, 3, 5, 6, 7, 15, 42, 64]);
}

#[test]
fn max_heap() {
    let mut heap = VecHeap::<i32, MaxHeap>::new();
    heap.push(3);
    heap.push(15);
    heap.push(1);
    heap.push(42);
    heap.push(7);
    heap.push(6);
    heap.push(5);
    heap.push(64);

    let mut data = Vec::new();
    while let Some(x) = heap.pop() {
        data.push(x);
    }
    assert_eq!(data, vec![64, 42, 15, 7, 6, 5, 3, 1]);
}

#[test]
fn unordered_heap() {
    let mut heap = IndexableHeap::<i32, MinHeap>::new();
    heap.push(3);
    heap.push(15);
    heap.push(1);
    let idx = heap.push(42);
    heap.push(7);
    heap.push(6);
    heap.push(5);
    heap.push(64);

    let mut data = Vec::new();
    for _ in 0..2 {
        data.push(heap.pop().unwrap());
    }

    let entry = heap.by_index_mut(idx);
    assert_eq!(*entry, 42);
    assert_eq!(entry.index(), idx);
    drop(entry);

    while let Some(x) = heap.pop() {
        data.push(x);
    }
    assert_eq!(data, vec![1, 3, 5, 6, 7, 15, 42, 64]);
}

#[test]
fn unordered_heap_mut() {
    let mut heap = IndexableHeap::<i32, MinHeap>::new();
    heap.push(3);
    heap.push(15);
    heap.push(1);
    let idx = heap.push(42);
    heap.push(7);
    heap.push(6);
    heap.push(5);
    heap.push(64);

    let mut data = Vec::new();
    for _ in 0..2 {
        data.push(heap.pop().unwrap());
    }

    *heap.by_index_mut(idx) = 1;

    while let Some(x) = heap.pop() {
        data.push(x);
    }
    assert_eq!(data, vec![1, 3, 1, 5, 6, 7, 15, 64]);
}
