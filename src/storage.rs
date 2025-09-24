use std::{mem::ManuallyDrop, ptr};

use crate::Position;

/// # Safety
///
/// The [`Self::parent`] and [`Self::child`] must never return the argument.
///
/// If any code creates a hole, this storage must never be passed to any safe function, that might try to read from the hole.
/// Unless specified otherwise, when a function receives a storage as parameter, it does not have any holes.
///
/// Any code that creates a hole must restore the hole before the storage is dropped.
pub unsafe trait Storage {
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// The whole item that is stored
    type Item;
    /// The key part of the item
    type Key;

    fn key(item: &Self::Item) -> &Self::Key;

    fn get(&self, pos: Position) -> &Self::Item;
    fn get_mut(&mut self, pos: Position) -> &mut Self::Item;
    fn get_key(&self, pos: Position) -> &Self::Key {
        Self::key(self.get(pos))
    }

    /// An value that represents the hole.
    /// It is used to put back data into the hole,
    /// which was possibly moved to another position.
    type Slot;
    fn slot_key(item: &Self::Slot) -> &Self::Key;
    /// Loads an element and creates a hole at `pos`
    ///
    /// # Safety
    ///
    /// * `pos` must not be a hole
    /// * Further "read" operations must never access the hole
    ///
    /// This operation creates a hole at `pos`
    ///
    /// By "read" operations, we mean (to be added, when more operations are added):
    /// * [`Self::get`]
    unsafe fn load(&self, pos: Position) -> ManuallyDrop<Self::Slot>;
    /// Stores an element into a hole at `pos`
    ///
    /// # Safety
    ///
    /// * `pos` must be a hole
    /// * the `item` must not be dropped
    ///
    /// This operation consumes the hole at `pos`
    unsafe fn store(&mut self, pos: Position, item: &mut ManuallyDrop<Self::Slot>);
    /// Moves element from `src` into a hole at `dst` without dropping.
    ///
    /// # Safety
    ///
    /// * `src` must not be a hole
    /// * `dst` must be a hole
    ///
    /// This operation moves the hole from `dst` to `src`
    unsafe fn move_element(&mut self, src: Position, dst: Position);
}

unsafe impl<T> Storage for [T] {
    fn len(&self) -> usize {
        self.len()
    }

    type Item = T;
    type Key = T;

    fn key(item: &Self::Item) -> &Self::Key {
        item
    }

    fn get(&self, pos: Position) -> &Self::Item {
        &self[pos]
    }

    fn get_mut(&mut self, pos: Position) -> &mut Self::Item {
        &mut self[pos]
    }

    type Slot = Self::Item;
    fn slot_key(item: &Self::Slot) -> &Self::Key {
        Self::key(item)
    }

    /// Loads an element and creates a hole at `pos`
    ///
    /// # Safety
    ///
    /// * `pos` must not be a hole
    /// * Further "read" operations must never access the hole
    ///
    /// This operation creates a hole at `pos`
    ///
    /// By "read" operations, we mean (to be added, when more operations are added):
    /// * [`Self::get`]
    unsafe fn load(&self, pos: Position) -> ManuallyDrop<Self::Item> {
        // SAFETY: pos is not a hole
        //         and we will never read the data from the hole
        let data = unsafe { ptr::read(&self[pos]) };
        ManuallyDrop::new(data)
    }

    /// Stores an element into a hole at `pos`
    ///
    /// # Safety
    ///
    /// * `pos` must be a hole
    /// * the `item` must not be dropped
    ///
    /// This operation consumes the hole at `pos`
    unsafe fn store(&mut self, pos: Position, item: &mut ManuallyDrop<Self::Item>) {
        // SAFETY: the `item` has not been dropped
        let item = unsafe { ManuallyDrop::take(item) };
        // SAFETY: pos is a hole
        unsafe { ptr::write(&mut self[pos], item) };
    }

    /// Moves element from `src` into a hole at `dst` without dropping.
    ///
    /// # Safety
    ///
    /// * `src` must not be a hole
    /// * `dst` must be a hole
    ///
    /// This operation moves the hole from `dst` to `src`
    unsafe fn move_element(&mut self, src: Position, dst: Position) {
        // SAFETY: src is not a hole and dst is a hole
        //         they cannot be equal, as the position cannot be a hole and not a hole at the same time
        unsafe { ptr::copy_nonoverlapping(&self[src], &mut self[dst], 1) };
        // dst was a hole, we filled it with a value - it is not a hole anymore
        // src is now a new hole
    }
}

unsafe impl<T> Storage for Vec<T> {
    fn len(&self) -> usize {
        self.as_slice().len()
    }

    type Item = T;

    type Key = <[T] as Storage>::Key;

    fn key(item: &Self::Item) -> &Self::Key {
        <[T]>::key(item)
    }

    fn get(&self, pos: Position) -> &Self::Item {
        Storage::get(self.as_slice(), pos)
    }

    fn get_mut(&mut self, pos: Position) -> &mut Self::Item {
        Storage::get_mut(self.as_mut_slice(), pos)
    }

    fn get_key(&self, pos: Position) -> &Self::Key {
        self.as_slice().get_key(pos)
    }

    type Slot = <[T] as Storage>::Slot;
    fn slot_key(item: &Self::Slot) -> &Self::Key {
        <[T]>::slot_key(item)
    }

    unsafe fn load(&self, pos: Position) -> ManuallyDrop<Self::Item> {
        // SAFETY: forwards to the underlying slice
        unsafe { self.as_slice().load(pos) }
    }

    unsafe fn store(&mut self, pos: Position, item: &mut ManuallyDrop<Self::Item>) {
        // SAFETY: forwards to the underlying slice
        unsafe { self.as_mut_slice().store(pos, item) }
    }

    unsafe fn move_element(&mut self, src: Position, dst: Position) {
        // SAFETY: forwards to the underlying slice
        unsafe { self.as_mut_slice().move_element(src, dst) }
    }
}
