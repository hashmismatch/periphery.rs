
pub use core::marker::PhantomData;
pub use core::iter;
pub use core::cell::RefCell;
pub use core::fmt;
pub use core::fmt::Debug;
pub use core::fmt::Write as FmtWrite;
pub use core::fmt::Error as FmtError;
pub use core::ops::Range;
pub use core::num::Wrapping;
pub use core::cmp::*;
pub use core::mem;
pub use core::intrinsics::write_bytes;
pub use core::ops::{Index, IndexMut, Deref};

pub use alloc::rc::Rc;
pub use alloc::arc::Arc;
pub use alloc::boxed::Box;

pub use collections::vec::Vec;
pub use collections::string::*;
pub use collections::slice::SliceConcatExt;
pub use collections::fmt::format as format_to_string;
pub use collections::fmt::{Display, Formatter};
pub use collections::borrow::Cow;
pub use collections::str::{from_utf8, FromStr};

pub use float_core::LocalFloat as CoreLocalFloat;

use core::intrinsics;

pub trait LocalFloat: Sized {
    fn ceil(self) -> Self;
    fn round(self) -> Self;
    fn floor(self) -> Self;
    fn abs(self) -> Self;
    fn powf(self, n: f32) -> Self;
}

impl LocalFloat for f32 {
	#[inline]
    fn ceil(self) -> f32 {
        unsafe { intrinsics::ceilf32(self) }
    }

	#[inline]
    fn round(self) -> f32 {
        unsafe { intrinsics::roundf32(self) }
    }

    #[inline]
    fn floor(self) -> f32 {
        unsafe { intrinsics::floorf32(self) }
    }

    #[inline]
    fn abs(self) -> f32 {
        unsafe { intrinsics::fabsf32(self) }
    }

    #[inline]
    fn powf(self, n: f32) -> f32 {
        unsafe { intrinsics::powf32(self, n) }
    }
}
