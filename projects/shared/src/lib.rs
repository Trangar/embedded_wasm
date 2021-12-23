#![no_std]

#[cfg(feature = "num_derive")]
use dep_num_derive::FromPrimitive;

#[derive(Debug)]
#[cfg_attr(feature = "num_derive", derive(FromPrimitive))]
#[repr(i32)]
pub enum LedIndex {
    D1,
    D2,
    D3,
    D4,
    D5,

    Unknown = i32::MAX,
}
