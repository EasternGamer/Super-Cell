use std::cell::UnsafeCell;
use std::cmp::Ordering;
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
#[cfg(feature = "serde")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Modified version of `Cell`
#[repr(transparent)]
pub struct SuperCell<T : ?Sized> {
    value : UnsafeCell<T>
}

impl <T> SuperCell<T> {
    #[inline]
    pub const fn new(value : T) -> Self {
        Self {
            value : UnsafeCell::new(value)
        }
    }

    #[inline]
    pub fn get_mut(&self) -> &mut T {
        unsafe { &mut (*self.value.get()) }
    }

    #[inline]
    pub fn get(&self) -> &T {
        unsafe { &(*self.value.get()) }
    }
}

impl<T> SuperCell<[T]> {

    pub fn as_slice_of_cells(&self) -> &[SuperCell<T>] {
        // SAFETY: `SuperCell<T>` has the same memory layout as `T`.
        unsafe { &*(self as *const SuperCell<[T]> as *const [SuperCell<T>]) }
    }
}

impl<T, const N: usize> SuperCell<[T; N]> {
    /// Returns a `&[SuperCell<T>; N]` from a `&SuperCell<[T; N]>`
    ///
    /// # Examples
    ///
    /// ```
    ///
    /// use super_cell::SuperCell;
    /// let mut array: [i32; 3] = [1, 2, 3];
    /// let cell_array: &SuperCell<[i32; 3]> = SuperCell::from_mut(&mut array);
    /// let array_cell: &[SuperCell<i32>; 3] = cell_array.as_array_of_cells();
    /// ```
    pub fn as_array_of_cells(&self) -> &[SuperCell<T>; N] {
        // SAFETY: `Cell<T>` has the same memory layout as `T`.
        unsafe { &*(self as *const SuperCell<[T; N]> as *const [SuperCell<T>; N]) }
    }
}

impl<T: Eq> Eq for SuperCell<T> {}
impl<T: PartialEq> PartialEq<Self> for SuperCell<T> {
    #[allow(dead_code)]
    fn eq(&self, other: &Self) -> bool {
        self.get_mut().eq(&other.get_mut())
    }
}
impl<T: PartialOrd> PartialOrd<Self> for SuperCell<T> {
    #[allow(dead_code)]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.get_mut().partial_cmp(&other.get_mut())
    }
}
impl<T : Ord> Ord for SuperCell<T> {
    #[allow(dead_code)]
    fn cmp(&self, other: &Self) -> Ordering {
        self.get_mut().cmp(&other.get_mut())
    }
}
impl <T : Default> Default for SuperCell<T> {
    #[allow(dead_code)]
    fn default() -> Self {
        SuperCell::new(T::default())
    }
}
impl<T: Clone> Clone for SuperCell<T> {
    #[allow(dead_code)]
    fn clone(&self) -> Self {
        SuperCell::new(self.get().clone())
    }
}
impl<T: Hash> Hash for SuperCell<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.get_mut().hash(state)
    }
}
impl <T: Debug> Debug for SuperCell<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.get_mut().fmt(f)
    }
}
impl <T: Display> Display for SuperCell<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.get_mut().fmt(f)
    }
}
unsafe impl <T> Sync for SuperCell<T> {}
unsafe impl <T> Send for SuperCell<T> {}

#[cfg(feature = "serde")]
impl <T : Serialize> Serialize for SuperCell<T> {
    fn serialize<S : Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.get_mut().serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl <'de,T : Deserialize<'de>> Deserialize<'de> for SuperCell<T> {
    fn deserialize<D : Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        T::deserialize(deserializer).map(SuperCell::new)
    }
}

#[cfg(test)]
mod tests {
    use std::thread;
    use std::thread::sleep;
    use std::time::Duration;
    use super::*;
    
    const TEST_COUNT : usize = 5;
    static COUNT: SuperCell<usize> = SuperCell::new(0);

    struct Test {
        x : usize,
        list : Vec<usize>
    }

    #[test]
    fn mutability_primitive() {
        let result = SuperCell::new(10);
        *result.get_mut() = 11;
        assert_eq!(*result.get(), 11);
        assert_eq!(*result.get_mut(), 11);
        println!("{}/{TEST_COUNT}: Mutability for Primitive Successful", *COUNT.get());
    }

    #[test]
    fn mutability_struct() {
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
        assert_eq!(result.get().x, 100);
        assert_eq!(result.get().list, list);

        assert_eq!(result.get_mut().x, 100);
        assert_eq!(result.get_mut().list, list);
        *COUNT.get_mut() += 1;
        println!("{}/{TEST_COUNT}: Mutability for Complex Struct Successful", *COUNT.get());
    }

    #[test]
    fn arrays() {
        let result = SuperCell::new([10;10]);
        let cells = result.as_array_of_cells();
        for cell in cells {
            *cell.get_mut() = 9;
        }
        for value in result.get() {
            assert_eq!(*value, 9)
        }
        *COUNT.get_mut() += 1;
        println!("{}/{TEST_COUNT}: Mutability for Cells as Arrays Successful", *COUNT.get());
    }

    #[test]
    fn async_mutability() {
        let result = SuperCell::new(10);
        thread::scope(|x| {
            let reference = result.get_mut();
            let handle = x.spawn(|| {
                sleep(Duration::from_millis(10));
                *reference = 11;
            });
            assert_eq!(*result.get(), 10);
            assert_eq!(*result.get_mut(), 10);
            handle.join().expect("Failed to join thread!");
            assert_eq!(*result.get(), 11);
            assert_eq!(*result.get_mut(), 11);
        });
        assert_eq!(*result.get(), 11);
        assert_eq!(*result.get_mut(), 11);
        *COUNT.get_mut() += 1;
        println!("{}/{TEST_COUNT}: Async Mutability for Cells as Arrays Successful", *COUNT.get());
    }

    #[test]
    fn multiple_mutable_references() {
        let result = SuperCell::new(10);
        let ref1 = result.get_mut();
        let ref2 = result.get_mut();
        let ref3 = result.get();
        *ref1 = 11;
        assert_eq!(*ref1, *ref2);
        assert_eq!(*ref3, *ref2);
        *COUNT.get_mut() += 1;
        println!("{}/{TEST_COUNT}: Multiple Mutability for Cell Successful", *COUNT.get());
    }
}
