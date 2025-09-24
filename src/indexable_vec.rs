use std::{
    fmt,
    marker::PhantomData,
    mem::{self, ManuallyDrop},
    ptr,
};

use crate::Position;

type RawIdx = usize;

/// An opaque handle to an element of type `T`.
///
/// This is returned by [`IndexableHeap::push`] and can be used to access
/// the element later via [`IndexableHeap::by_index_mut`], even after
/// the heap has been reordered by other operations.
///
/// The index is opaque and should not be inspected directly. It's designed
/// to be used only with the heap that created it.
///
/// If the element is removed from the heap, the index becomes invalid.
/// Even when the element is then pushed back, the index might be different.
///
/// When indexing with an invalid index, the heap might, might not panic.
/// Since the heap reuses indices, indexing with stale index might just return some unrelated element.
///
/// # Examples
///
/// ```
/// use mheap::{IndexableHeap, MaxHeap};
///
/// let mut heap = IndexableHeap::<i32, MaxHeap>::new();
/// let idx = heap.push(42);
/// heap.push(10);
/// heap.push(100);
///
/// // Later, we can still access the element by its index
/// *heap.by_index_mut(idx) = 50;
/// assert_eq!(heap.pop(), Some(100)); // 100 is still the max
/// assert_eq!(heap.pop(), Some(50)); // Our modified element
/// assert_eq!(heap.pop(), Some(10));
/// ```
///
/// See [`IndexableHeap`] for details
///
/// [`IndexableHeap`]: crate::IndexableHeap
/// [`IndexableHeap::push`]: crate::IndexableHeap::push
/// [`IndexableHeap::by_index_mut`]: crate::IndexableHeap::by_index_mut
pub struct Idx<T>(RawIdx, PhantomData<T>);

impl<T> Idx<T> {
    fn new(index: RawIdx) -> Self {
        Self(index, PhantomData)
    }

    fn index(&self) -> usize {
        self.0
    }
}

impl<T> Clone for Idx<T> {
    fn clone(&self) -> Self {
        Self(self.0, PhantomData)
    }
}

impl<T> Copy for Idx<T> {}

impl<T> PartialEq for Idx<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T> Eq for Idx<T> {}

impl<T> fmt::Debug for Idx<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Idx<{}>({})", std::any::type_name::<T>(), self.0)
    }
}

pub(crate) struct IndexableVec<T> {
    data: Vec<(T, Idx<T>)>,
    position: SkipList,
}

impl<T> IndexableVec<T> {
    pub(crate) const fn new() -> Self {
        Self {
            data: Vec::new(),
            position: SkipList::new(),
        }
    }

    pub(crate) fn with_capacity(capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
            position: SkipList::with_capacity(capacity),
        }
    }

    pub(crate) const fn len(&self) -> usize {
        self.data.len()
    }

    pub(crate) const fn capacity(&self) -> usize {
        self.data.capacity()
    }

    pub(crate) const fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub(crate) fn get(&self, pos: Position) -> &T {
        &self.data[pos].0
    }

    pub(crate) fn get_mut(&mut self, pos: Position) -> &mut T {
        &mut self.data[pos].0
    }

    pub(crate) fn push(&mut self, item: T) -> Idx<T> {
        let pos = self.data.len();
        let index = Idx::new(self.position.add(pos));
        self.data.push((item, index));
        index
    }

    pub(crate) fn pop(&mut self) -> Option<T> {
        let (item, index) = self.data.pop()?;
        // SAFETY: structure invariant
        unsafe { self.assert_index(self.data.len(), index) };
        self.position.remove(index.index());
        Some(item)
    }

    pub(crate) fn swap_remove(&mut self, pos: Position) -> T {
        let (item, index) = self.data.swap_remove(pos);
        // SAFETY: structure invariant
        unsafe { self.assert_index(pos, index) };
        self.position.remove(index.index());
        item
    }

    pub(crate) fn reserve(&mut self, additional: usize) {
        self.data.reserve(additional);
        self.position.reserve(additional);
    }

    pub(crate) fn reserve_exact(&mut self, additional: usize) {
        self.data.reserve_exact(additional);
        self.position.reserve_exact(additional);
    }

    pub(crate) fn shrink_to_fit(&mut self) {
        self.data.shrink_to_fit();
        self.position.shrink_to_fit();
    }

    pub(crate) fn shrink_to(&mut self, min_capacity: usize) {
        self.data.shrink_to(min_capacity);
        self.position.shrink_to(min_capacity);
    }

    fn record_position(&mut self, pos: Position) {
        let index = self.pos_to_index(pos);
        self.position.set(index.index(), pos);
    }

    pub(crate) fn index_to_pos(&self, index: Idx<T>) -> Position {
        let pos = self.position.get(index.index());
        // SAFETY: position map contains only valid positions
        unsafe { self.assert_pos(index, pos) };
        pos
    }

    pub(crate) fn pos_to_index(&self, pos: Position) -> Idx<T> {
        let index = self.data[pos].1;
        // SAFETY: data contains only valid indices
        unsafe { self.assert_index(pos, index) };
        return index;
    }

    /// Must ensure index is valid
    unsafe fn assert_index(&self, pos: Position, index: Idx<T>) {
        if !self.position.is_valid(index.index()) {
            if cfg!(debug_assertions) {
                panic!("position {pos} contains invalid index {}", index.index());
            }
            unsafe {
                std::hint::unreachable_unchecked();
            }
        }
    }

    /// Must ensure pos is valid
    unsafe fn assert_pos(&self, index: Idx<T>, pos: Position) {
        if pos >= self.data.len() {
            if cfg!(debug_assertions) {
                panic!(
                    "index {} resolved to invalid position {pos}; data length is {}",
                    index.index(),
                    self.data.len()
                );
            }
            unsafe {
                std::hint::unreachable_unchecked();
            }
        }
    }
}

unsafe impl<T> crate::storage::Storage for IndexableVec<T> {
    fn len(&self) -> usize {
        self.data.len()
    }

    type Item = T;

    type Key = T;

    fn key(item: &Self::Item) -> &Self::Key {
        item
    }

    fn get(&self, pos: Position) -> &Self::Item {
        self.get(pos)
    }

    fn get_mut(&mut self, pos: Position) -> &mut Self::Item {
        self.get_mut(pos)
    }

    type Slot = (T, Idx<T>);

    fn slot_key(item: &Self::Slot) -> &Self::Key {
        &item.0
    }

    unsafe fn load(&self, pos: Position) -> ManuallyDrop<Self::Slot> {
        ManuallyDrop::new(unsafe { ptr::read(&self.data[pos]) })
    }

    unsafe fn store(&mut self, pos: Position, item: &mut ManuallyDrop<Self::Slot>) {
        unsafe { ptr::write(&mut self.data[pos], ManuallyDrop::take(item)) };
        self.record_position(pos);
    }

    unsafe fn move_element(&mut self, src: Position, dst: Position) {
        unsafe { ptr::copy_nonoverlapping(&self.data[src], &mut self.data[dst], 1) };
        self.record_position(dst);
    }
}

struct SkipList {
    data: Vec<SkipEntry>,
    first_skip: NextSkip,
}

struct SkipEntry(usize);
#[derive(Clone, Copy)]
struct NextSkip(usize);

enum SkipEntryRepr {
    Data(Position),
    Skip { next_idx: NextSkip },
}

impl NextSkip {
    const NONE: Self = Self(0);

    fn some(next_idx: usize) -> Self {
        Self(next_idx + 1)
    }

    fn get(&self) -> Option<usize> {
        self.0.checked_sub(1)
    }
}

impl SkipEntry {
    const SKIP_BIT: usize = isize::MIN as usize;

    fn from_pos(pos: Position) -> Option<Self> {
        if pos & Self::SKIP_BIT == 0 {
            Some(Self(pos))
        } else {
            None
        }
    }

    fn from_skip(next_idx: NextSkip) -> Self {
        Self(next_idx.0 | Self::SKIP_BIT)
    }

    fn repr(&self) -> SkipEntryRepr {
        if self.0 & Self::SKIP_BIT == 0 {
            SkipEntryRepr::Data(self.0)
        } else {
            SkipEntryRepr::Skip {
                next_idx: NextSkip(self.0 & !Self::SKIP_BIT),
            }
        }
    }

    fn expect_skip(&self) -> NextSkip {
        return match self.repr() {
            SkipEntryRepr::Skip { next_idx } => next_idx,
            SkipEntryRepr::Data(_) => handle_data(),
        };

        #[cold]
        #[inline(never)]
        fn handle_data() -> ! {
            panic!("expected skip");
        }
    }

    fn is_data(&self) -> bool {
        matches!(self.repr(), SkipEntryRepr::Data(_))
    }

    fn expect_data(&self) -> Position {
        return match self.repr() {
            SkipEntryRepr::Data(data) => data,
            SkipEntryRepr::Skip { .. } => handle_skip(),
        };

        #[cold]
        #[inline(never)]
        fn handle_skip() -> ! {
            panic!("expected data");
        }
    }
}

impl SkipList {
    const fn new() -> Self {
        Self {
            data: Vec::new(),
            first_skip: NextSkip::NONE,
        }
    }

    fn with_capacity(capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
            first_skip: NextSkip::NONE,
        }
    }

    fn reserve(&mut self, additional: usize) {
        self.data.reserve(additional);
    }

    fn reserve_exact(&mut self, additional: usize) {
        self.data.reserve_exact(additional);
    }

    fn shrink_to_fit(&mut self) {
        self.data.shrink_to_fit();
    }

    fn shrink_to(&mut self, min_capacity: usize) {
        self.data.shrink_to(min_capacity);
    }

    fn add(&mut self, pos: Position) -> RawIdx {
        let pos = SkipEntry::from_pos(pos).unwrap();
        if let Some(index) = self.first_skip.get() {
            // SAFETY: all skip entries must always be valid
            unsafe { self.assert_index(index) };
            let entry = mem::replace(&mut self.data[index], pos);
            self.first_skip = entry.expect_skip();
            index
        } else {
            let index = self.data.len();
            self.data.push(pos);
            index
        }
    }

    fn is_valid(&self, index: RawIdx) -> bool {
        self.data.get(index).is_some_and(|it| it.is_data())
    }

    fn get(&self, index: RawIdx) -> Position {
        self.data[index].expect_data()
    }

    fn set(&mut self, index: RawIdx, pos: Position) {
        self.data[index].expect_data();
        self.data[index] = SkipEntry::from_pos(pos).unwrap();
    }

    fn remove(&mut self, index: RawIdx) -> Position {
        if index == self.data.len() - 1 {
            // We do not need to touch `self.first_skip`, since it cannot point to the last element (it is data, not skip)
            self.data.pop().unwrap().expect_data()
        } else {
            let new_skip = SkipEntry::from_skip(self.first_skip);
            let pos = mem::replace(&mut self.data[index], new_skip).expect_data();
            self.first_skip = NextSkip::some(index);
            pos
        }
    }

    /// Must ensure index is valid
    unsafe fn assert_index(&self, index: RawIdx) {
        if index >= self.data.len() {
            if cfg!(debug_assertions) {
                panic!("index {index} is out of bounds");
            }
            unsafe {
                std::hint::unreachable_unchecked();
            }
        }
    }
}
