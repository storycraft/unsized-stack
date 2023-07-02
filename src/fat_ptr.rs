/*
 * Created on Sun Jul 02 2023
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use core::mem;

#[derive(Debug)]
#[repr(C)]
pub struct FatPtr {
    ptr: *const (),
    metadata: usize,
}

impl FatPtr {
    pub const fn new<T>(ptr: *const T, metadata: usize) -> Self {
        Self {
            ptr: ptr as _,
            metadata,
        }
    }

    pub const fn ptr(&self) -> *const () {
        self.ptr
    }

    pub const fn metadata(&self) -> usize {
        self.metadata
    }
}

impl Clone for FatPtr {
    fn clone(&self) -> Self {
        Self {
            ptr: self.ptr,
            metadata: self.metadata,
        }
    }
}

impl Copy for FatPtr {}

#[repr(C)]
union FatPtrRepr<T: ?Sized> {
    pub ptr_const: *const T,
    pub fat_ptr: FatPtr,
}

pub const fn check_valid<T: ?Sized>() {
    if mem::size_of::<*const T>() != mem::size_of::<FatPtr>() {
        panic!("Type is not valid DST");
    }
}

pub const fn compose<T: ?Sized>(fat_ptr: FatPtr) -> *const T {

    unsafe { FatPtrRepr { fat_ptr }.ptr_const }
}

pub const fn decompose<T: ?Sized>(fat_ptr: *const T) -> FatPtr {
    check_valid::<T>();

    unsafe { FatPtrRepr { ptr_const: fat_ptr }.fat_ptr }
}

#[cfg(test)]
mod tests {
    use super::decompose;

    #[test]
    fn test() {
        let fat_ptr = decompose(&[0_usize] as *const [usize]);
        assert_eq!(fat_ptr.metadata(), 1);
    }
}
