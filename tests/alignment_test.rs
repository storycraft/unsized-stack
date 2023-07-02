/*
 * Created on Mon Jul 03 2023
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use std::fmt::Debug;
use unsized_stack::UnsizedStack;

#[derive(Debug)]
#[repr(align(32))]
pub struct A(u32);

#[derive(Debug)]
#[repr(align(128))]
pub struct B(u32);

#[derive(Debug)]
#[repr(align(512))]
pub struct C(u32);

#[derive(Debug)]
#[repr(align(1024))]
pub struct D;

#[test]
pub fn alignment_test() {
    let mut stack = UnsizedStack::<dyn Debug>::new();

    stack.push(A(0), |item| item);
    stack.push(B(0), |item| item);
    stack.push(C(0), |item| item);
    stack.push(D, |item| item); // ZST

    assert_eq!(stack.buf_layout().align(), 512);
}
