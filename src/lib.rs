#![doc = include_str!("../readme.md")]
// Unstable features
#![feature(ptr_metadata, unsize)]

pub mod iter;

use core::{
    fmt::Debug,
    marker::Unsize,
    mem::{self},
    ops::{Index, IndexMut},
    ptr::{self, DynMetadata, Pointee},
    slice,
};
use iter::{Iter, IterMut};

pub struct TraitStack<T: ?Sized + Pointee<Metadata = DynMetadata<T>>> {
    data: Vec<u8>,

    table: Vec<(usize, DynMetadata<T>)>
}

impl<T: ?Sized + Pointee<Metadata = DynMetadata<T>>> TraitStack<T> {
    pub const fn new() -> Self {
        Self {
            data: Vec::new(),
            table: Vec::new()
        }
    }

    #[inline]
    pub fn data_len(&self) -> usize {
        self.data.len()
    }

    #[inline]
    pub fn data_capacity(&self) -> usize {
        self.data.capacity()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.table.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.table.is_empty()
    }

    #[inline]
    pub fn data_as_ptr(&self) -> *const u8 {
        self.data.as_ptr()
    }

    pub fn get<'a>(&'a self, index: usize) -> Option<&'a T> {
        // SAFETY: Manually constructed reference have valid lifetime
        unsafe { Some(&*self.get_ptr(index)?) }
    }

    pub fn get_mut<'a>(&'a mut self, index: usize) -> Option<&'a mut T> {
        // SAFETY: Manually constructed reference have valid lifetime
        unsafe { Some(&mut *(self.get_ptr(index)? as *mut T)) }
    }

    unsafe fn get_ptr(&self, index: usize) -> Option<*const T> {
        let (offset, metadata) = *self.table.get(index)?;

        Some(self.dyn_ptr_from(offset, metadata))
    }

    unsafe fn dyn_ptr_from(&self, offset: usize, metadata: DynMetadata<T>) -> *const T {
        let data = self.data.as_ptr().add(offset) as _;

        ptr::from_raw_parts(data, metadata)
    }

    pub fn push<I: Unsize<T>>(&mut self, item: I) {
        let (data, metadata) = (&item as *const T).to_raw_parts();

        let offset = self.data.len();

        // SAFETY: item is moved to data and original is forgotten.
        self.data
            .extend_from_slice(unsafe { slice::from_raw_parts(data as _, mem::size_of::<I>()) });
        mem::forget(item);

        self.table.push((offset, metadata));
    }

    pub fn pop(&mut self) -> Option<()> {
        let (offset, metadata) = self.table.pop()?;
        let data = unsafe { self.dyn_ptr_from(offset, metadata) };

        // SAFETY: Data get removed after destructor
        unsafe { ptr::drop_in_place(data as *mut T) };
        self.data.truncate(offset);

        Some(())
    }

    pub fn last(&self) -> Option<&T> {
        self.get(self.len() - 1)
    }

    pub fn last_mut(&mut self) -> Option<&mut T> {
        self.get_mut(self.len() - 1)
    }

    pub fn iter(&self) -> Iter<T> {
        Iter {
            ptr: self.data.as_ptr(),
            table_iter: self.table.iter(),
        }
    }

    pub fn iter_mut(&mut self) -> IterMut<T> {
        IterMut {
            ptr: self.data.as_mut_ptr(),
            table_iter: self.table.iter(),
        }
    }

    pub fn clear(&mut self) {
        for (offset, metadata) in &self.table {
            // SAFETY: Data and table cleared after drop
            unsafe {
                ptr::drop_in_place(self.dyn_ptr_from(*offset, *metadata) as *mut T);
            }
        }

        self.table.clear();
        self.data.clear();
    }
}

impl<T: ?Sized + Pointee<Metadata = DynMetadata<T>>> Drop for TraitStack<T> {
    fn drop(&mut self) {
        for (offset, metadata) in &self.table {
            // SAFETY: Data and table invalid after destructor
            unsafe {
                ptr::drop_in_place(self.dyn_ptr_from(*offset, *metadata) as *mut T);
            }
        }
    }
}

impl<T: ?Sized + Pointee<Metadata = DynMetadata<T>>> Default for TraitStack<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: ?Sized + Pointee<Metadata = DynMetadata<T>>> Index<usize> for TraitStack<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        self.get(index).unwrap()
    }
}

impl<T: ?Sized + Pointee<Metadata = DynMetadata<T>>> IndexMut<usize> for TraitStack<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.get_mut(index).unwrap()
    }
}

// SAFETY: All data stored in [TraitStack] is Send
unsafe impl<T: ?Sized + Pointee<Metadata = DynMetadata<T>> + Send> Send for TraitStack<T> {}

// SAFETY: All data stored in [TraitStack] is Sync
unsafe impl<T: ?Sized + Pointee<Metadata = DynMetadata<T>> + Sync> Sync for TraitStack<T> {}

impl<'a, T: ?Sized + Pointee<Metadata = DynMetadata<T>>> IntoIterator for &'a TraitStack<T> {
    type Item = &'a T;

    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T: ?Sized + Pointee<Metadata = DynMetadata<T>>> IntoIterator for &'a mut TraitStack<T> {
    type Item = &'a mut T;

    type IntoIter = IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<T: ?Sized + Pointee<Metadata = DynMetadata<T>> + Debug> Debug for TraitStack<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self).finish()
    }
}
