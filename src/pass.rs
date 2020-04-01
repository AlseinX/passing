use std::{
    cell::UnsafeCell,
    fmt::Debug,
    ops::{Deref, DerefMut},
    sync::{Arc, MutexGuard},
};
use Locator::*;

enum Locator<'a, T> {
    ByOwned(T),
    ByRef(&'a T),
    ByRefMut(&'a mut T),
    ByLock(MutexGuard<'a, T>),
}

pub trait Relocate<'a, T> {
    fn relocate(&mut self, v: T);
}

#[derive(Clone)]
pub struct Pass<'a, T> {
    locator: Arc<UnsafeCell<Locator<'a, T>>>,
}

pub struct PassMut<'a, T> {
    locator: Arc<UnsafeCell<Locator<'a, T>>>,
}

impl<T> PassMut<'_, T> {
    #[inline]
    fn from_locator<'a>(locator: Locator<'a, T>) -> PassMut<'a, T> {
        PassMut::<'a, T> {
            locator: Arc::new(UnsafeCell::new(locator)),
        }
    }
}

impl<T> Pass<'_, T> {
    #[inline]
    fn from_locator<'a>(locator: Locator<'a, T>) -> Pass<'a, T> {
        Pass::<'a, T> {
            locator: Arc::new(UnsafeCell::new(locator)),
        }
    }
}

impl<'a, T> From<(T,)> for PassMut<'a, T> {
    fn from(v: (T,)) -> Self {
        Self::from_locator(ByOwned(v.0))
    }
}

impl<'a, T> From<&'a mut T> for PassMut<'a, T> {
    fn from(v: &'a mut T) -> Self {
        Self::from_locator(ByRefMut(v))
    }
}

impl<'a, T> From<MutexGuard<'a, T>> for PassMut<'a, T> {
    fn from(v: MutexGuard<'a, T>) -> Self {
        Self::from_locator(ByLock(v))
    }
}

impl<'a, T> From<(T,)> for Pass<'a, T> {
    fn from(v: (T,)) -> Self {
        Self::from_locator(ByOwned(v.0))
    }
}

impl<'a, T> From<&'a T> for Pass<'a, T> {
    fn from(v: &'a T) -> Self {
        Self::from_locator(ByRef(v))
    }
}

impl<'a, T> From<&'a mut T> for Pass<'a, T> {
    fn from(v: &'a mut T) -> Self {
        Self::from_locator(ByRefMut(v))
    }
}

impl<'a, T> From<MutexGuard<'a, T>> for Pass<'a, T> {
    fn from(v: MutexGuard<'a, T>) -> Self {
        Self::from_locator(ByLock(v))
    }
}

impl<'a, T> From<PassMut<'a, T>> for Pass<'a, T> {
    fn from(v: PassMut<'a, T>) -> Self {
        Self {
            locator: v.locator.clone(),
        }
    }
}

impl<T> Relocate<'_, (T,)> for PassMut<'_, T> {
    fn relocate(&mut self, v: (T,)) {
        self.locator = Arc::new(UnsafeCell::new(ByOwned(v.0)));
    }
}

impl<'a, 'b: 'a, T> Relocate<'a, &'b mut T> for PassMut<'a, T> {
    fn relocate(&mut self, v: &'b mut T) {
        self.locator = Arc::new(UnsafeCell::new(ByRefMut(v)));
    }
}

impl<'a, 'b: 'a, T> Relocate<'a, MutexGuard<'b, T>> for PassMut<'a, T> {
    fn relocate(&mut self, v: MutexGuard<'b, T>) {
        self.locator = Arc::new(UnsafeCell::new(ByLock(v)));
    }
}

impl<T> Relocate<'_, (T,)> for Pass<'_, T> {
    fn relocate(&mut self, v: (T,)) {
        self.locator = Arc::new(UnsafeCell::new(ByOwned(v.0)));
    }
}

impl<'a, 'b: 'a, T> Relocate<'a, &'b T> for Pass<'a, T> {
    fn relocate(&mut self, v: &'b T) {
        self.locator = Arc::new(UnsafeCell::new(ByRef(v)));
    }
}

impl<'a, 'b: 'a, T> Relocate<'a, &'b mut T> for Pass<'a, T> {
    fn relocate(&mut self, v: &'b mut T) {
        self.locator = Arc::new(UnsafeCell::new(ByRefMut(v)));
    }
}

impl<'a, 'b: 'a, T> Relocate<'a, MutexGuard<'b, T>> for Pass<'a, T> {
    fn relocate(&mut self, v: MutexGuard<'b, T>) {
        self.locator = Arc::new(UnsafeCell::new(ByLock(v)));
    }
}

impl<'a, T> Deref for PassMut<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe {
            match &*self.locator.get() {
                ByOwned(v) => &*v,
                ByRef(v) => *v,
                ByRefMut(v) => *v,
                ByLock(v) => &*v,
            }
        }
    }
}

impl<'a, T> DerefMut for PassMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            match &mut *self.locator.get() {
                ByOwned(v) => &mut *v,
                ByRefMut(v) => *v,
                ByLock(v) => v.deref_mut(),
                _ => panic!(),
            }
        }
    }
}

impl<'a, T> Deref for Pass<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe {
            match &*self.locator.get() {
                ByOwned(v) => &*v,
                ByRef(v) => *v,
                ByRefMut(v) => *v,
                ByLock(v) => &*v,
            }
        }
    }
}

impl<T> Debug for PassMut<'_, T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        (&**self).fmt(f)
    }
}

impl<T> Debug for Pass<'_, T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        (&**self).fmt(f)
    }
}
