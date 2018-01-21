
use core::intrinsics;

type c_float = f32;

extern {
	fn acosf(n: c_float) -> c_float;
	fn asinf(n: c_float) -> c_float;
    fn atan2f(a: c_float, b: c_float) -> c_float;
    fn atanf(n: c_float) -> c_float;
    fn coshf(n: c_float) -> c_float;
    fn sinhf(n: c_float) -> c_float;
    fn tanf(n: c_float) -> c_float;
    fn tanhf(n: c_float) -> c_float;    
}

pub trait LocalFloat: Sized {
    fn sqrt(self) -> Self;
    fn atan2(self, other: f32) -> f32;
    fn atan(self) -> f32;    
    fn asin(self) -> f32;
    fn acos(self) -> f32;
}

impl LocalFloat for f32 {
	#[inline]
    fn sqrt(self) -> f32 {
    	use core::f32::NAN;

        if self < 0.0 {
            NAN
        } else {
            unsafe { intrinsics::sqrtf32(self) }
        }
    }

    #[inline]
    fn atan2(self, other: f32) -> f32 {
        unsafe { atan2f(self, other) }
    }

    #[inline]
    fn atan(self) -> f32 {
        unsafe { atanf(self) }
    }

    #[inline]
    fn asin(self) -> f32 {
        unsafe { asinf(self) }
    }

    #[inline]
    fn acos(self) -> f32 {
        unsafe { acosf(self) }
    }
}
