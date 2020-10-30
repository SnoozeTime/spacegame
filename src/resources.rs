//! Container for resources, that can be any type. This is inspired by Shred and AnyMap.
//! AnyMap didn't fill my usecase as there is no way to borrow mutably 2 values for different
//! keys. (`get_mut(&mut self)`).
//!
//! This container is using interior mutability with `RefCell` to allow this usecase.
//! Downcasting trait does not work with pure rust so I am using a crate called `downcast_rs` to
//! do it.
//!
//!
//! How to user.
//! ```
//! use spacegame::resources::Resources;
//! let mut resources = Resources::new();
//! resources.insert(String::from("Bonjour"));
//! resources.insert(0u8);
//!
//! // Modify a value.
//! {
//!     let mut my_u8 = resources.fetch_mut::<u8>().unwrap();
//!     *my_u8 += 1;
//! }
//!
//! // Read a value and modify another value in the same scope.
//! {
//!      let my_u8 = resources.fetch::<u8>().unwrap();
//!      let mut my_string = resources.fetch_mut::<String>().unwrap();
//!      my_string.push_str("hhh");
//!      println!("{}", *my_u8);
//!  }
//!
//! ```
use downcast_rs::{impl_downcast, Downcast};
use std::any::{Any, TypeId};
use std::cell::{Ref, RefCell, RefMut};
use std::collections::HashMap;
use std::convert::AsRef;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

pub trait Resource: Any + 'static + Downcast {}
impl_downcast!(Resource);

impl<T> Resource for T where T: Any + 'static {}

pub struct Fetch<'a, T: 'static> {
    inner: Ref<'a, dyn Resource>,
    phantom: PhantomData<&'a T>,
}

impl<'a, T: 'static> Deref for Fetch<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.inner.downcast_ref::<T>().unwrap()
    }
}

pub struct FetchMut<'a, T: 'static> {
    inner: RefMut<'a, dyn Resource>,
    phantom: PhantomData<&'a T>,
}

impl<'a, T: 'static> Deref for FetchMut<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.inner.downcast_ref::<T>().unwrap()
    }
}

impl<'a, T: 'static> DerefMut for FetchMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner.downcast_mut::<T>().unwrap()
    }
}

#[derive(Default)]
pub struct Resources {
    inner: HashMap<TypeId, RefCell<Box<dyn Resource>>>,
}

impl Resources {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    /// Insert a new value for the type. It will replace the existing value
    /// if it exists.
    pub fn insert<T: Any>(&mut self, v: T) {
        self.inner
            .insert(TypeId::of::<T>(), RefCell::new(Box::new(v)));
    }

    /// Borrow data immutably from the map. Can panic if already borrowed mutably.
    pub fn fetch<T: Any + 'static>(&self) -> Option<Fetch<T>> {
        let cloned = {
            let ty = TypeId::of::<T>();
            match self.inner.get(&ty) {
                Some(v) => v,
                None => return None,
            }
        };
        let borrowed: Ref<Box<dyn Resource>> = cloned.borrow();
        Some(Fetch {
            inner: Ref::map(borrowed, Box::as_ref),
            phantom: PhantomData,
        })
    }

    /// Borrow data mutably from the map. Can panic if already borrowed
    pub fn fetch_mut<T: Any + 'static>(&self) -> Option<FetchMut<T>> {
        let cloned = {
            let ty = TypeId::of::<T>();
            match self.inner.get(&ty) {
                Some(v) => v,
                None => return None,
            }
        };

        // this panics if already borrowed.
        let borrowed = cloned.borrow_mut();
        Some(FetchMut {
            inner: RefMut::map(borrowed, Box::as_mut),
            phantom: PhantomData,
        })
    }
}
