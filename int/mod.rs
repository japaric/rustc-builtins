
pub mod mul;
pub mod sdiv;
pub mod shift;
pub mod udiv;

/// Trait for some basic operations on integers
pub trait Int {
    /// Returns the bitwidth of the int type
    fn bits() -> u32;
}

// TODO: Once i128/u128 support lands, we'll want to add impls for those as well
impl Int for u32 {
    fn bits() -> u32 {
        32
    }
}
impl Int for i32 {
    fn bits() -> u32 {
        32
    }
}
impl Int for u64 {
    fn bits() -> u32 {
        64
    }
}
impl Int for i64 {
    fn bits() -> u32 {
        64
    }
}

/// Trait to convert an integer to/from smaller parts
pub trait LargeInt {
    type LowHalf;
    type HighHalf;

    fn low(self) -> Self::LowHalf;
    fn high(self) -> Self::HighHalf;
    fn from_parts(low: Self::LowHalf, high: Self::HighHalf) -> Self;
}

// TODO: Once i128/u128 support lands, we'll want to add impls for those as well
impl LargeInt for u64 {
    type LowHalf = u32;
    type HighHalf = u32;

    fn low(self) -> u32 {
        self as u32
    }
    fn high(self) -> u32 {
        (self >> 32) as u32
    }
    fn from_parts(low: u32, high: u32) -> u64 {
        low as u64 | ((high as u64) << 32)
    }
}
impl LargeInt for i64 {
    type LowHalf = u32;
    type HighHalf = i32;

    fn low(self) -> u32 {
        self as u32
    }
    fn high(self) -> i32 {
        (self >> 32) as i32
    }
    fn from_parts(low: u32, high: i32) -> i64 {
        low as i64 | ((high as i64) << 32)
    }
}
