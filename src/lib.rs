/*
 * Created on Sun Jul 02 2023
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

#![doc = include_str!("../readme.md")]

mod fat_ptr;
pub mod iter;
pub mod raw;

use iter::{Iter, IterMut};
use raw::RawUnsizedStack;
use std::{
    alloc::Layout,
    fmt::Debug,
    marker::PhantomData,
    ops::{Index, IndexMut},
    ptr::NonNull,
};

pub struct UnsizedStack<T: ?Sized> {
    raw: RawUnsizedStack<T>,
}

impl<T: ?Sized> UnsizedStack<T> {
    pub const fn new() -> Self {
        Self {
            raw: RawUnsizedStack::new(),
        }
    }

    pub const fn bytes_occupied(&self) -> usize {
        self.raw.bytes_occupied()
    }

    pub const fn buf_layout(&self) -> Layout {
        self.raw.buf_layout()
    }

    pub const fn buf_ptr(&self) -> NonNull<u8> {
        self.raw.buf_ptr()
    }

    pub fn last(&self) -> Option<&T> {
        self.raw.with_table(|table| table.last())
    }

    pub fn last_mut(&mut self) -> Option<&mut T> {
        self.raw.with_table_mut(|table| table.last())
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        self.raw.with_table(|table| table.get(index))
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.raw.with_table_mut(|table| table.get(index))
    }

    pub fn push<I>(&mut self, item: I, coercion: fn(&I) -> &T) {
        self.raw.push(item, coercion)
    }

    pub fn len(&self) -> usize {
        self.raw.table().len()
    }

    pub fn is_empty(&self) -> bool {
        self.raw.table().is_empty()
    }

    pub fn pop(&mut self) -> Option<()> {
        self.raw.pop()
    }

    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            base: self.raw.buf_ptr().as_ptr(),
            table_iter: self.raw.table().iter(),
            _phantom: PhantomData,
        }
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        IterMut {
            base: self.raw.buf_ptr().as_ptr(),
            table_iter: self.raw.table().iter(),
            _phantom: PhantomData,
        }
    }

    pub fn clear(&mut self) {
        self.raw.clear();
    }
}

impl<T: ?Sized> Default for UnsizedStack<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, T: ?Sized> IntoIterator for &'a UnsizedStack<T> {
    type Item = &'a T;

    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T: ?Sized> IntoIterator for &'a mut UnsizedStack<T> {
    type Item = &'a mut T;

    type IntoIter = IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<T: ?Sized> Index<usize> for UnsizedStack<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        self.get(index).unwrap()
    }
}

impl<T: ?Sized> IndexMut<usize> for UnsizedStack<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.get_mut(index).unwrap()
    }
}

impl<T: ?Sized + Debug> Debug for UnsizedStack<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}
