/*
 * Created on Sat Jun 18 2022
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use core::slice;
use std::marker::PhantomData;

use crate::raw::{self, TableItem};

pub struct Iter<'a, T: ?Sized> {
    pub(crate) base: *const u8,
    pub(crate) table_iter: slice::Iter<'a, TableItem>,
    pub(crate) _phantom: PhantomData<&'a T>,
}

impl<'a, T: ?Sized> Iter<'a, T> {
    fn with_iter(
        &mut self,
        func: impl FnOnce(&mut slice::Iter<'a, TableItem>) -> Option<&'a TableItem>,
    ) -> Option<&'a T> {
        Some(unsafe { &*raw::compose::<T>(self.base, *func(&mut self.table_iter)?) })
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
    pub(crate) base: *const u8,
    pub(crate) table_iter: slice::Iter<'a, TableItem>,
    pub(crate) _phantom: PhantomData<&'a mut T>,
}

impl<'a, T: ?Sized> IterMut<'a, T> {
    fn with_iter(
        &mut self,
        func: impl FnOnce(&mut slice::Iter<'a, TableItem>) -> Option<&'a TableItem>,
    ) -> Option<&'a mut T> {
        Some(unsafe { &mut *raw::compose::<T>(self.base, *func(&mut self.table_iter)?).cast_mut() })
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
