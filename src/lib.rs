#![doc = include_str!("../readme.md")]
// Unstable features
#![feature(ptr_metadata, unsize)]

mod fat_ptr;
pub mod iter;
pub mod raw;

use core::{
    fmt::Debug,
    marker::Unsize,
    mem,
    ops::{Index, IndexMut},
    ptr::{self, DynMetadata, Pointee},
};
use iter::{Iter, IterMut};
use std::alloc;
use std::{alloc::Layout, ptr::NonNull};

#[repr(align(16))]
struct DefaultBuffer;

pub struct TraitStack<T: ?Sized + Pointee<Metadata = DynMetadata<T>>> {
    buf: NonNull<u8>,
    buf_layout: Layout,
    buf_occupied: usize,

    table: Vec<(usize, DynMetadata<T>)>,
}

impl<T: ?Sized + Pointee<Metadata = DynMetadata<T>>> TraitStack<T> {
    pub const DEFAULT_ALIGN: usize = mem::align_of::<DefaultBuffer>();

    pub const fn new() -> Self {
        Self {
            buf: NonNull::<DefaultBuffer>::dangling().cast(),
            buf_layout: Layout::new::<DefaultBuffer>(),
            buf_occupied: 0,
            table: Vec::new(),
        }
    }

    pub const fn buf_layout(&self) -> Layout {
        self.buf_layout
    }

    #[inline]
    pub fn table_capacity(&self) -> usize {
        self.table.capacity()
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
    pub fn buf_ptr(&self) -> *const u8 {
        self.buf.as_ptr().cast_const()
    }

    #[inline]
    pub fn get(&self, index: usize) -> Option<&T> {
        // SAFETY: Manually constructed reference have valid lifetime
        unsafe { Some(&*self.get_ptr(index)?) }
    }

    #[inline]
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        // SAFETY: Manually constructed reference have valid lifetime
        unsafe { Some(&mut *(self.get_ptr(index)? as *mut T)) }
    }

    #[inline]
    unsafe fn get_ptr(&self, index: usize) -> Option<*const T> {
        let (offset, metadata) = *self.table.get(index)?;

        Some(self.dyn_ptr_from(offset, metadata))
    }

    #[inline]
    unsafe fn dyn_ptr_from(&self, offset: usize, metadata: DynMetadata<T>) -> *const T {
        ptr::from_raw_parts(self.buf.as_ptr().add(offset) as _, metadata)
    }

    fn reserve_space_for(&mut self, value_layout: Layout) -> usize {
        let padding = self.buf_occupied % value_layout.align();
        let offset = self.buf_occupied + padding;

        let new_buf_layout = Layout::from_size_align(
            (self.buf_occupied + padding + value_layout.size()).next_power_of_two(),
            self.buf_layout.align().max(value_layout.align()),
        )
        .unwrap();

        if self.buf_layout != new_buf_layout {
            let new_buf = unsafe {
                if self.buf_layout.size() == 0 {
                    alloc::alloc(new_buf_layout)
                } else if self.buf_layout.align() != new_buf_layout.align() {
                    alloc::dealloc(self.buf.as_ptr(), self.buf_layout);
                    alloc::alloc(new_buf_layout)
                } else {
                    alloc::realloc(self.buf.as_ptr(), self.buf_layout, new_buf_layout.size())
                }
            };

            self.buf = NonNull::new(new_buf).unwrap();
            self.buf_layout = new_buf_layout;
        }
        self.buf_occupied += padding + value_layout.size();

        return offset;
    }

    pub fn push<I: Unsize<T>>(&mut self, mut item: I) {
        let (data, metadata) = (&mut item as *mut T).to_raw_parts();

        let item_layout = Layout::new::<I>();
        let offset = self.reserve_space_for(item_layout);

        // SAFETY: item is moved to data and original is forgotten.
        unsafe {
            ptr::copy_nonoverlapping(
                data as *mut u8,
                self.buf.as_ptr().add(offset),
                item_layout.size(),
            )
        };
        mem::forget(item);

        self.table.push((offset, metadata));
        self.buf_occupied += item_layout.size();
    }

    pub fn pop(&mut self) -> Option<()> {
        let (offset, metadata) = self.table.pop()?;
        let data = unsafe { self.dyn_ptr_from(offset, metadata) };

        // SAFETY: Data get removed after destructor
        unsafe { ptr::drop_in_place(data as *mut T) };
        self.buf_occupied = offset;

        Some(())
    }

    pub fn truncate(&mut self, len: usize) {
        if len >= self.table.len() {
            return;
        }

        let data_start_offset = self.table[len].0;

        for i in len..self.len() {
            let (offset, metadata) = self.table[i];

            // SAFETY: Data get removed after destructor
            unsafe {
                let data = self.dyn_ptr_from(offset, metadata);
                ptr::drop_in_place(data as *mut T)
            };
        }

        self.table.truncate(len);
        self.buf_occupied = data_start_offset;
    }

    #[inline]
    pub fn last(&self) -> Option<&T> {
        self.get(self.len() - 1)
    }

    #[inline]
    pub fn last_mut(&mut self) -> Option<&mut T> {
        self.get_mut(self.len() - 1)
    }

    #[inline]
    pub fn iter(&self) -> Iter<T> {
        Iter {
            ptr: self.buf.as_ptr(),
            table_iter: self.table.iter(),
        }
    }

    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<T> {
        IterMut {
            ptr: self.buf.as_ptr(),
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
        self.buf_occupied = 0;
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

        if self.buf_layout.size() > 0 {
            unsafe {
                alloc::dealloc(self.buf.as_ptr(), self.buf_layout);
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
