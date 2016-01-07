// Copyright (c) 2015 The Robigalia Project Developers
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>,
// at your option. All files in the project carrying such
// notice may not be copied, modified, or distributed except
// according to those terms.

//! Higher level interfaces to seL4 kernel objects.
//!
//! The intent of this crate is to provide mechanism, not policy, so the general flavour is still
//! very low-level and architecture-specific details are not abstracted over. However, it should be
//! more convenient than the raw sel4-sys functions and no less performant (once optimized, of
//! course).
//!
//! **Note**: when method documentation says "this", it refers to the receiver of the thread, not
//! any global state.

#![no_std]
#![allow(stable_features, unused_features)]
#![feature(no_std, core_slice_ext)]
#![doc(html_root_url = "https://doc.robigalia.org/")]

extern crate sel4_sys;
use sel4_sys::*;

/// Canonical result type from invoking capabilities.
pub type Result = core::result::Result<(), Error>;

pub trait ToCap {
    /// Unwrap this object into its raw capability pointer.
    fn to_cap(&self) -> seL4_CPtr;
}

pub trait FromCap: Sized {
    /// Wrap a capability pointer with this type.
    ///
    /// Does no checking that the capability is to an object of the correct type.
    fn from_cap(cptr: seL4_CPtr) -> Self;
}

pub trait Allocatable {
    /// Allocate an object, using memory from the untyped memory object and storing the capability
    /// into `Window`.
    ///
    /// The number of objects to create is the `num_slots` field on the `Window`.
    fn create(untyped_memory: seL4_CPtr, dest: Window, size_bits: isize) -> Result;
}

impl ToCap for seL4_CPtr {
    #[inline(always)]
    fn to_cap(&self) -> seL4_CPtr {
        *self
    }
}

impl FromCap for seL4_CPtr {
    #[inline(always)]
    fn from_cap(cptr: seL4_CPtr) -> Self {
        cptr
    }
}

macro_rules! cap_wrapper {
    ($($(#[$attr:meta])* : $name:ident $objtag:ident)*) => ($(
        cap_wrapper_inner!($(#[$attr])* : $name);
        impl ::Allocatable for $name {
            fn create(untyped_memory: ::sel4_sys::seL4_CPtr, dest: ::cspace::Window, size_bits: isize) -> ::Result {
                use ToCap;
                errcheck!(seL4_Untyped_Retype(untyped_memory, $objtag as isize, size_bits, dest.cnode.root.to_cap(),
                                    dest.cnode.index as isize, dest.cnode.depth as isize, dest.first_slot_idx as isize, dest.num_slots as isize));
            }
        }
    )*)
}

macro_rules! cap_wrapper_inner {
    ($($(#[$attr:meta])* : $name:ident)*) => ($(
        #[derive(Debug, Copy, Clone, PartialEq, Eq)]
        $(#[$attr])* pub struct $name {
            cptr: ::sel4_sys::seL4_CPtr,
        }
        impl ::ToCap for $name {
            #[inline(always)]
            fn to_cap(&self) -> ::sel4_sys::seL4_CPtr {
                self.cptr.to_cap()
            }
        }
        impl ::FromCap for $name {
            #[inline(always)]
            fn from_cap(cptr: ::sel4_sys::seL4_CPtr) -> Self {
                $name { cptr: cptr }
            }
        }
    )*)
}

macro_rules! errcheck {
    ($e:expr) => {
        if unsafe { $e } == 0 { return Ok(()) } else { return Err(::Error(::GoOn::CheckIPCBuf)) }
    }
}

/// An error occured.
///
/// Since seL4 stores error information in the IPC buffer, and copying that data is not free, to
/// inspect the details of the error you must call `.details()`.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Error(GoOn);

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum GoOn {
    CheckIPCBuf,
    TooMuchData,
    TooManyCaps, // WouldBlock,
}

pub use cspace::{CNode, SlotRef, Badge};
pub use error::{ErrorDetails, LookupFailureKind};
pub use endpoint::{Endpoint, RecvToken};
pub use notification::Notification;
pub use thread::{Thread, ThreadConfiguration};
pub use domain::DomainSet;
pub use irq::{IRQControl, IRQHandler};

/// Sets the destination for capability transfer to the given slot.
pub fn set_cap_destination(slot: SlotRef) {
    unsafe {
        let buf = seL4_GetIPCBuffer();
        (*buf).receiveCNode = slot.root.to_cap();
        (*buf).receiveIndex = slot.index as seL4_Word;
        (*buf).receiveDepth = slot.depth as seL4_Word;
    }
}

/// Gets the current capability transfer destination.
pub fn get_cap_destination() -> SlotRef {
    unsafe {
        let buf = seL4_GetIPCBuffer();
        SlotRef::new(CNode::from_cap((*buf).receiveCNode),
                     (*buf).receiveIndex,
                     (*buf).receiveDepth as u8)
    }
}

/// Yield the remainder of the current timeslice back to the scheduler.
#[inline(always)]
pub fn yield_now() {
    unsafe {
        seL4_Yield();
    }
}

mod cspace;
mod error;
mod endpoint;
mod notification;
mod thread;
mod domain;
mod irq;
#[cfg(all(target_arch = "x86", target_pointer_width = "32"))]
mod arch {
    include!("arch/x86.rs");
}
#[cfg(all(target_arch = "arm", target_pointer_width = "32"))]
mod arch {
    include!("arch/arm.rs");
}

pub use arch::*;
