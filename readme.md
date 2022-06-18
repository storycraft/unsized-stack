# TraitStack
Store trait object and vtable using two Vec.

Provides efficient heterogeneous list when the elements don't need to get resorted.

## Comparion to `Vec<Box<dyn Trait>>`
`Vec<Box<dyn Trait>>` makes allocation per element. Inserting many objects can hurt performance due to allocation.

`TraitStack` stack object to its inner vector. It makes moving element by index impossible and only can be used as stack. but very fast on insertion.

See diagram for detailed structure.

## Diagram
![diagram](images/diagram.svg)

## no_std
No support.

## Example
```rust
let mut stack = TraitStack::<dyn Debug>::new();
stack.push("str");
stack.push(1);
stack.push(28342.2);
println!("{:?}", stack); // Print ["str", 1, 28342.2]
```