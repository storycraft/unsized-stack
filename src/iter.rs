/*
 * Created on Sat Jun 18 2022
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use core::slice;
use std::{marker::PhantomData, ptr::NonNull};

use crate::raw::{self, RawUnsizedStack, TableItem};

pub struct Iter<'a, T: ?Sized> {
    base: NonNull<u8>,
    table_iter: slice::Iter<'a, TableItem>,
    _phantom: PhantomData<&'a T>,
}

impl<'a, T: ?Sized> Iter<'a, T> {
    pub fn new(raw: &'a RawUnsizedStack<T>) -> Self {
        Self {
            base: raw.buf_ptr(),
            table_iter: raw.table().iter(),
            _phantom: PhantomData,
        }
    }

    fn with_iter(
        &mut self,
        func: impl FnOnce(&mut slice::Iter<'a, TableItem>) -> Option<&'a TableItem>,
    ) -> Option<&'a T> {
        // Safety: pointer created with [`TableItem`] from table
        Some(unsafe { &*raw::compose::<T>(self.base.as_ptr(), *func(&mut self.table_iter)?) })
    }
}

impl<'a, T: ?Sized> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.with_iter(|iter| iter.next())
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.table_iter.size_hint()
    }

    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.table_iter.count()
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.with_iter(|iter| iter.nth(n))
    }
}

impl<'a, T: 'a + ?Sized> DoubleEndedIterator for Iter<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.with_iter(slice::Iter::next_back)
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.with_iter(|iter| iter.nth_back(n))
    }
}

impl<'a, T: ?Sized> ExactSizeIterator for Iter<'a, T> {}

pub struct IterMut<'a, T: ?Sized> {
    base: NonNull<u8>,
    table_iter: slice::Iter<'a, TableItem>,
    _phantom: PhantomData<&'a mut T>,
}

impl<'a, T: ?Sized> IterMut<'a, T> {
    pub fn new(raw: &'a mut RawUnsizedStack<T>) -> Self {
        Self {
            base: raw.buf_ptr(),
            table_iter: raw.table().iter(),
            _phantom: PhantomData,
        }
    }

    fn with_iter(
        &mut self,
        func: impl FnOnce(&mut slice::Iter<'a, TableItem>) -> Option<&'a TableItem>,
    ) -> Option<&'a mut T> {
        // Safety: pointer created with [`TableItem`] from table
        Some(unsafe {
            &mut *raw::compose::<T>(self.base.as_ptr(), *func(&mut self.table_iter)?).cast_mut()
        })
    }
}

impl<'a, T: 'a + ?Sized> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        self.with_iter(slice::Iter::next)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.table_iter.size_hint()
    }

    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.table_iter.count()
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.with_iter(|iter| iter.nth(n))
    }
}

impl<'a, T: ?Sized> DoubleEndedIterator for IterMut<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.with_iter(slice::Iter::next_back)
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.with_iter(|iter| iter.nth_back(n))
    }
}

impl<'a, T: ?Sized> ExactSizeIterator for IterMut<'a, T> {}
