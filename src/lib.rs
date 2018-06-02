// Copyright 2018 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

//! # System-Uri Library
//! [Project github page](https://github.com/maidsafe/system_uri)

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/maidsafe/QA/master/Images/maidsafe_logo.png",
    html_favicon_url = "http://maidsafe.net/img/favicon.ico",
    html_root_url = "http://maidsafe.github.io/system_uri"
)]
// For explanation of lint checks, run `rustc -W help` or see
// https://github.
// com/maidsafe/QA/blob/master/Documentation/Rust%20Lint%20Checks.md

// inspired by
// https://github.
// com/feross/webtorrent-desktop/blob/4bb2056bc9c1a421815b97d03ffed512575dfde0/src/main/handlers.js
#![forbid(
    exceeding_bitshifts, mutable_transmutes, no_mangle_const_items, unknown_crate_types, warnings
)]
#![deny(
    bad_style, deprecated, improper_ctypes, missing_docs, non_shorthand_field_patterns,
    overflowing_literals, plugin_as_library, private_no_mangle_fns, private_no_mangle_statics,
    stable_features, unconditional_recursion, unknown_lints, unsafe_code, unused, unused_allocation,
    unused_attributes, unused_comparisons, unused_features, unused_parens, while_true
)]
#![warn(
    trivial_casts, trivial_numeric_casts, unused_extern_crates, unused_import_braces,
    unused_qualifications, unused_results
)]
// TODO: Remove `renamed_and_removed_lints` once
// https://github.com/rust-lang-nursery/error-chain/pull/246 has been fixed.
#![allow(
    box_pointers, missing_copy_implementations, missing_debug_implementations,
    variant_size_differences, renamed_and_removed_lints
)]
#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", plugin(clippy))]
#![cfg_attr(
    feature = "clippy", deny(clippy, unicode_not_nfc, wrong_pub_self_convention, option_unwrap_used)
)]
#![cfg_attr(feature = "clippy", allow(use_debug, too_many_arguments, needless_return))]
// `error_chain!` can recurse deeply
#![recursion_limit = "1024"]

#[macro_use]
extern crate error_chain;

#[cfg(any(target_os = "macos", feature = "ffi"))]
extern crate libc;

#[cfg(target_os = "linux")]
extern crate xdg_basedir;

#[cfg(feature = "ffi")]
extern crate ffi_utils;

mod app;
pub use app::App;

mod errors {
    use std::str::Utf8Error;

    error_chain! {
        types {
            Error, ErrorKind, ChainErr, Result;
        }

        foreign_links {
            Utf8Error(Utf8Error);
        }

        errors {
            /// The SystemURI error used to wrap problems
            SystemUriError(t: String) {
                description("System URI Error")
                display("Could not execute: {}", t)
            }
        }
    }
}

pub use errors::Error as SystemUriError;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use linux::{install, open};

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub use windows::{install, open};

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub use macos::{install, open};

/// Foreign Function call Interface to use this library
#[cfg(feature = "ffi")]
pub mod ffi;
