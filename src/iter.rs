/*
 * Created on Sat Jun 18 2022
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use core::{slice, ptr::{self, DynMetadata, Pointee}};

pub struct Iter<'a, T: ?Sized + Pointee<Metadata = DynMetadata<T>>> {
    pub(crate) ptr: *const u8,
    pub(crate) table_iter: slice::Iter<'a, (usize, DynMetadata<T>)>,
}

impl<'a, T: ?Sized + Pointee<Metadata = DynMetadata<T>>> Iter<'a, T> {
    unsafe fn item_at(&self, offset: usize, metadata: DynMetadata<T>) -> &'a T {
        &*(ptr::from_raw_parts(self.ptr.add(offset) as _, metadata) as *const T)
    }
}

impl<'a, T: ?Sized + Pointee<Metadata = DynMetadata<T>>> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let (offset, metadata) = self.table_iter.next()?;

        // SAFETY: Pointer is offseted using valid offset
        Some(unsafe { self.item_at(*offset, *metadata) })
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
        let (offset, metadata) = self.table_iter.nth(n)?;

        return Some(unsafe { self.item_at(*offset, *metadata) });
    }
}

impl<'a, T: ?Sized + Pointee<Metadata = DynMetadata<T>>> DoubleEndedIterator for Iter<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let (offset, metadata) = self.table_iter.next_back()?;

        // SAFETY: Pointer is offseted using valid offset
        Some(unsafe { self.item_at(*offset, *metadata) })
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        let (offset, metadata) = self.table_iter.nth_back(n)?;

        // SAFETY: Pointer is offseted using valid offset
        return Some(unsafe { self.item_at(*offset, *metadata) });
    }
}

impl<'a, T: ?Sized + Pointee<Metadata = DynMetadata<T>>> ExactSizeIterator for Iter<'a, T> {}

pub struct IterMut<'a, T: ?Sized + Pointee<Metadata = DynMetadata<T>>> {
    pub(crate) ptr: *const u8,
    pub(crate) table_iter: slice::Iter<'a, (usize, DynMetadata<T>)>,
}

impl<'a, T: ?Sized + Pointee<Metadata = DynMetadata<T>>> IterMut<'a, T> {
    unsafe fn item_at(&mut self, offset: usize, metadata: DynMetadata<T>) -> &'a mut T {
        &mut *(ptr::from_raw_parts::<T>(self.ptr.add(offset) as _, metadata) as *mut T)
    }
}

impl<'a, T: ?Sized + Pointee<Metadata = DynMetadata<T>>> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        let (offset, metadata) = self.table_iter.next()?;

        // SAFETY: Pointer is offseted using valid offset
        Some(unsafe { self.item_at(*offset, *metadata) })
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
        let (offset, metadata) = self.table_iter.nth(n)?;

        return Some(unsafe { self.item_at(*offset, *metadata) });
    }
}

impl<'a, T: ?Sized + Pointee<Metadata = DynMetadata<T>>> DoubleEndedIterator for IterMut<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let (offset, metadata) = self.table_iter.next_back()?;

        // SAFETY: Pointer is offseted using valid offset
        Some(unsafe { self.item_at(*offset, *metadata) })
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        let (offset, metadata) = self.table_iter.nth_back(n)?;

        // SAFETY: Pointer is offseted using valid offset
        return Some(unsafe { self.item_at(*offset, *metadata) });
    }
}

impl<'a, T: ?Sized + Pointee<Metadata = DynMetadata<T>>> ExactSizeIterator for IterMut<'a, T> {}
