# UnsizedStack
Store unboxed dst objects

Provides efficient heterogeneous list when the elements don't need to get resorted.

## Comparion to `Vec<Box<dyn Trait>>`
`Vec<Box<dyn Trait>>` makes allocation per element. Inserting many objects can hurt performance due to allocation.

`UnsizedStack` stack object to its inner storage. It makes moving element by index impossible and only can be used as stack. but very fast on insertion.

See diagram for detailed structure.

## Diagram
![diagram](images/diagram.svg)

## no_std
No support.

## Example
```rust
use unsized_stack::UnsizedStack;
use std::fmt::Debug;

let mut stack = UnsizedStack::<dyn Debug>::new();
stack.push("str", |item| item as _);
stack.push(1, |item| item as _);
stack.push(28342.2, |item| item as _);
dbg!(stack); // Print stack = ["str", 1, 28342.2]
```