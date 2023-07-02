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

    stack.pop();
    stack.pop();
    stack.pop();
    stack.pop();

    assert_eq!(stack.len(), 0);
}

#[test]
fn str_stack_test() {
    let mut stack = UnsizedStack::<str>::new();

    stack.push("String", |item| *item as _);
    stack.push("ASDF", |item| *item as _);

    dbg!(&stack);
    assert_eq!(stack.len(), 2);

    stack.pop();
    stack.pop();

    assert_eq!(stack.len(), 0);
}
