#![feature(debug_closure_helpers)]

// TODO : add a no-std feature to deactivate the parts that need std

use std::{
    collections::{BTreeMap, HashMap},
    convert::Infallible,
    env::Args,
    ffi::{CStr, CString, OsStr, OsString},
    fmt::{self, Debug},
    ops::Deref,
    path::{Path, PathBuf}, rc::Rc, sync::Arc,
};

pub use debug_with_context_macros::DebugWithContext;

pub trait DebugWithContext<C> {
    fn fmt_with_context(&self, f: &mut fmt::Formatter, context: &C) -> fmt::Result;
}

pub struct DebugWrapContext<'a, C, T> {
    value: &'a T,
    context: &'a C,
}

impl<'a, C, T> DebugWrapContext<'a, C, T> {
    pub fn new(value: &'a T, context: &'a C) -> DebugWrapContext<'a, C, T> {
        DebugWrapContext { value, context }
    }
}

impl<'a, C, T> Debug for DebugWrapContext<'a, C, T>
where
    T: DebugWithContext<C>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.value.fmt_with_context(f, self.context)
    }
}

macro_rules! debug_with_context_debug {
    ($t:ty) => {
        impl <C> DebugWithContext<C> for $t {
            fn fmt_with_context(&self, f: &mut fmt::Formatter, _context: &C) -> fmt::Result {
                write!(f, "{:?}", self)
            }
        }
    };
    ($t1:ty, $($ts:ty),+) => {
        debug_with_context_debug!($t1);
        debug_with_context_debug!($($ts),+);
    }
}

// TODO : add types here ?
debug_with_context_debug!(
    bool,
    u8,
    i8,
    u16,
    i16,
    u32,
    i32,
    u64,
    i64,
    u128,
    i128,
    f32,
    f64,
    usize,
    isize,
    char,
    &str,
    String,
    ()
);
debug_with_context_debug!(
    Infallible, Args, CStr, CString, OsStr, OsString, Path, PathBuf
);

/*impl <'a, C> DebugWithContext<C> for Chars<'a> {
    fn fmt_with_context(&self, f: &mut fmt::Formatter, _context: &C) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}*/

// TODO : use a macro to create tuples for example to a bigger size
impl<C, T1, T2> DebugWithContext<C> for (T1, T2)
where
    T1: DebugWithContext<C>,
    T2: DebugWithContext<C>,
{
    fn fmt_with_context(&self, f: &mut fmt::Formatter, context: &C) -> fmt::Result {
        f.debug_tuple("")
            .field_with(|fmt| self.0.fmt_with_context(fmt, context))
            .field_with(|fmt| self.1.fmt_with_context(fmt, context))
            .finish()
    }
}

#[inline]
fn fmt_with_context_collection<C, Col, T>(
    col: &Col,
    f: &mut fmt::Formatter,
    context: &C,
) -> fmt::Result
where
    Col: Deref<Target = [T]>,
    T: DebugWithContext<C>,
{
    f.debug_list()
        .entries(col.iter().map(|item| DebugWrapContext {
            value: item,
            context,
        }))
        .finish()
}

impl<C, T> DebugWithContext<C> for Vec<T>
where
    T: DebugWithContext<C>,
{
    fn fmt_with_context(&self, f: &mut fmt::Formatter, context: &C) -> fmt::Result {
        fmt_with_context_collection(self, f, context)
    }
}

impl<C, T> DebugWithContext<C> for &[T]
where
    T: DebugWithContext<C>,
{
    fn fmt_with_context(&self, f: &mut fmt::Formatter, context: &C) -> fmt::Result {
        fmt_with_context_collection(self, f, context)
    }
}

impl<C, T> DebugWithContext<C> for Box<[T]>
where
    T: DebugWithContext<C>,
{
    fn fmt_with_context(&self, f: &mut fmt::Formatter, context: &C) -> fmt::Result {
        fmt_with_context_collection(self, f, context)
    }
}

impl<C, T> DebugWithContext<C> for Rc<[T]>
where
    T: DebugWithContext<C>,
{
    fn fmt_with_context(&self, f: &mut fmt::Formatter, context: &C) -> fmt::Result {
        fmt_with_context_collection(self, f, context)
    }
}

impl<C, T> DebugWithContext<C> for Arc<[T]>
where
    T: DebugWithContext<C>,
{
    fn fmt_with_context(&self, f: &mut fmt::Formatter, context: &C) -> fmt::Result {
        fmt_with_context_collection(self, f, context)
    }
}

impl<C, T> DebugWithContext<C> for Option<T>
where
    T: DebugWithContext<C>,
{
    fn fmt_with_context(&self, f: &mut fmt::Formatter, context: &C) -> fmt::Result {
        match self {
            Some(s) => s.fmt_with_context(f, context),
            None => None::<()>.fmt(f),
        }
    }
}

impl<C, K, V, S> DebugWithContext<C> for HashMap<K, V, S>
where
    K: DebugWithContext<C>,
    V: DebugWithContext<C>,
{
    fn fmt_with_context(&self, f: &mut fmt::Formatter, context: &C) -> fmt::Result {
        f.debug_map()
            .entries(self.iter().map(|(k, v)| {
                (
                    DebugWrapContext::new(k, context),
                    DebugWrapContext::new(v, context),
                )
            }))
            .finish()
    }
}

impl<C, K, V> DebugWithContext<C> for BTreeMap<K, V>
where
    K: DebugWithContext<C>,
    V: DebugWithContext<C>,
{
    fn fmt_with_context(&self, f: &mut fmt::Formatter, context: &C) -> fmt::Result {
        f.debug_map()
            .entries(self.iter().map(|(k, v)| {
                (
                    DebugWrapContext::new(k, context),
                    DebugWrapContext::new(v, context),
                )
            }))
            .finish()
    }
}

impl<C, T> DebugWithContext<C> for &'_ T
where
    T: DebugWithContext<C>,
{
    fn fmt_with_context(&self, f: &mut fmt::Formatter, context: &C) -> fmt::Result {
        (*self).fmt_with_context(f, context)
    }
}

impl<C, T> DebugWithContext<C> for &'_ mut T
where
    T: DebugWithContext<C>,
{
    fn fmt_with_context(&self, f: &mut fmt::Formatter, context: &C) -> fmt::Result {
        (**self).fmt_with_context(f, context)
    }
}
