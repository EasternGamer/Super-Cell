# Super Cell
A super (unsafe) that modifies `Cell` type to be more convenient and thus introduce potentially undefined behaviour for mutable types.
Undefined behaviour possible:
- Multiple mutable references from references leading to possible race-conditions
- Concurrent modification in separate threads with no guarantee of order
- ...and probably more.

This leaves the safe-ness in the hands of the programmer, but generally in using this in my own projects, I have not come across a time when this has caused an issue.

## Usage
```rust
use super_cell::*;

fn main() {
    // Mutable primitive
    let result = SuperCell::new(10);
    *result.get_mut() = 11;
    assert_eq!(*result.get(), 11); // OK
    assert_eq!(*result.get_mut(), 11); // OK

    // Mutable Complex Struct
    let result = SuperCell::new(Test {
        x: 0,
        list: vec![],
    });
    let mutable = result.get_mut();
    let mut list = vec![];
    mutable.x = 100;
    for i in 0..100 {
        mutable.list.push(i);
        list.push(i);
    }
    assert_eq!(result.get().x, 100); // OK
    assert_eq!(result.get().list, list); // OK
    assert_eq!(result.get_mut().x, 100); // OK
    assert_eq!(result.get_mut().list, list); // OK
    
    // Mutable Parallel/Async
    let result = SuperCell::new(10);
    thread::scope(|x| {
        let reference = result.get_mut();
        let handle = x.spawn(|| {
            sleep(Duration::from_millis(10));
            *reference = 11;
        });
        assert_eq!(*result.get(), 10); // OK
        assert_eq!(*result.get_mut(), 10); // OK
        handle.join().expect("Failed to join thread!");
        assert_eq!(*result.get(), 11); // OK
        assert_eq!(*result.get_mut(), 11); // OK
    });
    assert_eq!(*result.get(), 11); // OK
    assert_eq!(*result.get_mut(), 11); // OK
}
```
