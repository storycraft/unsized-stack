use trait_stack::TraitStack;

use std::fmt::Debug;

#[test]
fn trait_stack_test() {
    let mut stack = TraitStack::<dyn Debug>::new();

    stack.push("str");
    stack.push(1);
    stack.push(28342.2);
    stack.push("String".to_string());

    assert_eq!(stack.len(), 4);

    stack.pop();
    stack.pop();
    stack.pop();
    stack.pop();
    
    assert_eq!(stack.len(), 0);
}
