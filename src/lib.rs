// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! An implementation of an efficient O(n) median filter.

#![cfg_attr(all(not(feature = "std"), not(test)), no_std)]
#![cfg_attr(feature = "missing_mpl", feature(plugin))]
#![cfg_attr(feature = "missing_mpl", plugin(missing_mpl))]
#![cfg_attr(feature = "missing_mpl", deny(missing_mpl))]
#![warn(missing_docs)]

#[cfg(all(not(feature = "std"), not(test)))]
extern crate core as std;

extern crate generic_array;

#[cfg(feature = "std")]
pub mod heap;

pub mod stack;

#[cfg(feature = "std")]
pub use heap::*;
#[cfg(not(feature = "std"))]
pub use stack::*;
