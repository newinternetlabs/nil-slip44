// Copyright (c) 2025 New Internet Labs Limited
//
// SPDX-License-Identifier: MIT

#![doc = include_str!("../README.md")]
#[cfg(doctest)]
pub struct ReadmeDoctests;

mod coin;
mod coins_macro;
pub use coin::*;
