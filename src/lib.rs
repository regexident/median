#![cfg_attr(not(any(feature = "std", test)), no_std)]
#![warn(missing_docs)]
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! An implementation of an efficient O(n) median filter.

#[cfg(not(any(feature = "std", test)))]
extern crate core as std;

extern crate generic_array;

#[cfg(feature = "std")]
pub mod heap;

pub mod stack;

pub use heap::*;
