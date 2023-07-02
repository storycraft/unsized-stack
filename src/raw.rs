/*
 * Created on Sun Jul 02 2023
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use core::{
    alloc::Layout,
    marker::PhantomData,
    ptr::{self, NonNull},
};
use std::{
    alloc::{alloc, dealloc, realloc},
    fmt::Debug,
    mem,
};

use crate::fat_ptr::{self, FatPtr};

#[repr(align(16))]
struct DefaultBuffer;

pub struct RawUnsizedStack<T: ?Sized> {
    buf: NonNull<u8>,
    buf_layout: Layout,
    buf_occupied: usize,

    table: Vec<TableItem>,
    _phantom: PhantomData<T>,
}

impl<T: ?Sized> RawUnsizedStack<T> {
    pub const DEFAULT_ALIGN: usize = mem::align_of::<DefaultBuffer>();

    pub const fn new() -> Self {
        fat_ptr::check_valid::<T>();

        Self {
            buf: NonNull::dangling(),
            buf_layout: Layout::new::<DefaultBuffer>(),
            buf_occupied: 0,
            table: Vec::new(),
            _phantom: PhantomData,
        }
    }

    pub const fn bytes_occupied(&self) -> usize {
        self.buf_occupied
    }

    pub const fn buf_layout(&self) -> Layout {
        self.buf_layout
    }

    pub const fn buf_ptr(&self) -> NonNull<u8> {
        self.buf.cast()
    }

    pub fn table(&self) -> &[TableItem] {
        &self.table
    }

    pub fn push<I>(&mut self, item: I, coercion: fn(&I) -> &T) {
        let (item_layout, item_ptr) = {
            let coercion_ref = coercion(&item);
            (
                Layout::for_value(coercion_ref),
                fat_ptr::decompose(coercion_ref as *const _),
            )
        };

        if item_layout.size() == 0 {
            self.table.push(TableItem::new(
                Offset::Zst(item_layout.align()),
                item_ptr.metadata(),
            ));
            return;
        }

        let offset = {
            let padding = ((self.buf_occupied + item_layout.align() - 1)
                & !(item_layout.align() - 1))
                - self.buf_occupied;

            self.buf_occupied + padding
        };

        let new_buf_layout = Layout::from_size_align(
            (offset + item_layout.size())
                .next_power_of_two()
                .max(self.buf_layout.size()),
            item_layout.align().max(self.buf_layout.align()),
        )
        .unwrap();

        if new_buf_layout.align() != self.buf_layout.align() {
            self.buf = {
                if self.buf_layout.size() != 0 {
                    unsafe {
                        dealloc(self.buf.as_ptr(), self.buf_layout);
                    }
                }

                NonNull::new(unsafe { alloc(new_buf_layout) }).unwrap()
            };

            self.buf_layout = new_buf_layout;
        } else if new_buf_layout.size() > self.buf_layout.size() {
            self.buf = if self.buf_layout.size() == 0 {
                NonNull::new(unsafe { alloc(new_buf_layout) }).unwrap()
            } else {
                NonNull::new(unsafe {
                    realloc(self.buf.as_ptr(), self.buf_layout, new_buf_layout.size())
                })
                .unwrap()
            };

            self.buf_layout = new_buf_layout;
        }

        self.buf_occupied = offset + item_layout.size();
        unsafe {
            ptr::copy_nonoverlapping(
                item_ptr.ptr() as *const u8,
                self.buf.as_ptr().wrapping_add(offset),
                item_layout.size(),
            );
        }
        mem::forget(item);

        self.table
            .push(TableItem::new(Offset::Data(offset), item_ptr.metadata()));
    }

    pub fn pop(&mut self) -> Option<()> {
        let item = self.table.pop()?;
        unsafe {
            drop_item::<T>(self.buf.as_ptr(), item);
        }

        if let Offset::Data(offset) = item.offset {
            self.buf_occupied = offset;
        }

        Some(())
    }

    pub fn last(&self) -> Option<&T> {
        Some(unsafe { &*compose::<T>(self.buf.as_ptr(), *self.table.last()?) })
    }

    pub fn last_mut(&mut self) -> Option<&mut T> {
        Some(unsafe { &mut *compose::<T>(self.buf.as_ptr(), *self.table.last_mut()?).cast_mut() })
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        Some(unsafe { &*compose::<T>(self.buf.as_ptr(), *self.table.get(index)?) })
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        Some(unsafe { &mut *compose::<T>(self.buf.as_ptr(), *self.table().get(index)?).cast_mut() })
    }

    pub fn clear(&mut self) {
        self.table.drain(..).for_each(|item| unsafe {
            drop_item::<T>(self.buf.as_ptr(), item);
        });
    }
}

impl<T: ?Sized + Debug> Debug for RawUnsizedStack<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list()
            .entries(
                self.table
                    .iter()
                    .copied()
                    .map(|item| unsafe { &*compose::<T>(self.buf.as_ptr(), item) }),
            )
            .finish()
    }
}

impl<T: ?Sized> Drop for RawUnsizedStack<T> {
    fn drop(&mut self) {
        for item in self.table.iter().copied() {
            unsafe {
                drop_item::<T>(self.buf.as_ptr(), item);
            }
        }

        if self.buf_layout.size() > 0 {
            unsafe {
                dealloc(self.buf.as_ptr(), self.buf_layout);
            }
        }
    }
}

pub(crate) unsafe fn drop_item<T: ?Sized>(base: *const u8, item: TableItem) {
    ptr::drop_in_place(compose::<T>(base, item).cast_mut());
}

pub(crate) fn compose<T: ?Sized>(base: *const u8, item: TableItem) -> *const T {
    fat_ptr::compose::<T>(item.to_fat_ptr(base))
}

#[derive(Debug, Clone, Copy)]
pub enum Offset {
    Data(usize),
    Zst(usize),
}

#[derive(Debug, Clone, Copy)]
pub struct TableItem {
    pub offset: Offset,
    pub metadata: usize,
}

impl TableItem {
    pub const fn new(offset: Offset, metadata: usize) -> Self {
        Self { offset, metadata }
    }

    pub const fn to_fat_ptr(&self, base: *const u8) -> FatPtr {
        FatPtr::new(
            match self.offset {
                Offset::Data(offset) => base.wrapping_add(offset),
                Offset::Zst(align) => sptr::invalid(align),
            },
            self.metadata,
        )
    }
}
