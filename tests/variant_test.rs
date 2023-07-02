/*
 * Created on Mon Jul 03 2023
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use unsized_stack::UnsizedStack;

use std::fmt::Debug;

#[test]
fn trait_stack_test() {
    let mut stack = UnsizedStack::<dyn Debug>::new();

    stack.push("str", |item| item as _);
    stack.push(1, |item| item as _);
    stack.push(28342.2, |item| item as _);
    stack.push("String".to_string(), |item| item as _);

    dbg!(&stack);
    assert_eq!(stack.len(), 4);
}

#[test]
fn str_stack_test() {
    let mut stack = UnsizedStack::<str>::new();

    stack.push("", |item| item); // ZST
    stack.push("ASDF", |item| item);

    dbg!(&stack);
    assert_eq!(stack.len(), 2);
}

#[test]
fn slice_stack_test() {
    let mut stack = UnsizedStack::<[i32]>::new();

    stack.push([1, 2, 3, 4], |item| item);
    stack.push([5, 6], |item| item);
    stack.push([7, 8, 9], |item| item);

    dbg!(&stack);
    assert_eq!(stack.len(), 3);
}
