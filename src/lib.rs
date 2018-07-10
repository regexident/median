#![cfg_attr(not(any(feature = "std", test)), no_std)]
#![warn(missing_docs)]

//! An implementation of an efficient O(n) median filter.

#[cfg(not(any(feature = "std", test)))]
extern crate core as std;

extern crate generic_array;

#[cfg(feature = "std")]
pub mod heap;

pub mod stack;

pub use heap::*;
