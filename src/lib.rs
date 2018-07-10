// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! An implementation of an efficient O(n) median filter.

#![cfg_attr(feature = "missing_mpl", feature(plugin))]
#![cfg_attr(feature = "missing_mpl", plugin(missing_mpl))]
#![cfg_attr(feature = "missing_mpl", deny(missing_mpl))]

#![warn(missing_docs)]

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate core as std;

#[cfg(not(feature = "std"))]
#[macro_use]
extern crate std;

extern crate generic_array;

#[cfg(feature = "std")]
pub mod heap;

pub mod stack;

pub use heap::*;
