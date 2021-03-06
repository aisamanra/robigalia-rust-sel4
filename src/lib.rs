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
//! very low-level and architecture-specific details are not abstracted over.  However, it should
//! be more convenient than the raw sel4-sys functions and no less performant (once optimized, of
//! course).
//!
//! **Note**: when method documentation says "this", it refers to the receiver of the thread, not
//! any global state.

#![no_std]
#![allow(stable_features, unused_features)]
#![feature(no_std, core_slice_ext, const_fn)]
#![doc(html_root_url = "https://doc.robigalia.org/")]

extern crate sel4_sys;
use sel4_sys::{seL4_CPtr, seL4_GetIPCBuffer, seL4_Word, seL4_Yield};

#[macro_use]
mod macros;

mod alloc;
mod arch;
mod cspace;
mod domain;
mod endpoint;
mod error;
mod irq;
mod notification;
mod thread;

pub use alloc::ObjectAllocator;
pub use arch::*;
pub use cspace::{Badge, CNode, CNodeInfo, SlotRef, Window};
pub use domain::DomainSet;
pub use endpoint::{Endpoint, RecvToken};
pub use error::{ErrorDetails, LookupFailureKind};
pub use irq::{IRQControl, IRQHandler};
pub use notification::Notification;
pub use thread::{Thread, ThreadConfiguration};


// TODO: This should be a configuration option pulled from sel4 kernel config
pub const CONFIG_RETYPE_FAN_OUT_LIMIT: usize = 256;

/// Canonical result type from invoking capabilities.
pub type Result = core::result::Result<(), Error>;

pub trait ToCap {
    /// Unwrap this object into its raw capability pointer.
    fn to_cap(&self) -> seL4_CPtr;
}

pub trait Allocatable {
    /// Allocate an object, using memory from the untyped memory object and storing the capability
    /// into `Window`.
    ///
    /// The number of objects to create is the `num_slots` field on the `Window`.
    fn create(untyped_memory: seL4_CPtr, dest: Window, size_bits: seL4_Word) -> Result;
    fn object_size(size_bits: seL4_Word) -> isize;
}

impl ToCap for seL4_CPtr {
    #[inline(always)]
    fn to_cap(&self) -> seL4_CPtr {
        *self
    }
}

/// An error occured.
///
/// Since seL4 stores error information in the IPC buffer, and copying that data is not free, to
/// inspect the details of the error you must call `.details()`. The `Debug` implementation will do
/// this automatically, to aid debugging.
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Error(pub GoOn);

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum GoOn {
    CheckIPCBuf,
    TooMuchData,
    TooManyCaps,
//  WouldBlock,
}

impl core::fmt::Debug for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self.0 {
            GoOn::CheckIPCBuf => {
                match self.details() {
                    Some(deets) => write!(f, "{:?} ({})", deets, deets),
                    None => write!(f, "no error"),
                }
            }
            GoOn::TooMuchData => f.write_str("TooMuchData"),
            GoOn::TooManyCaps => f.write_str("TooManyCaps"),
        }
    }
}

/// Sets the (thread-local) destination for capability transfer to the given slot.
pub fn set_cap_destination(slot: SlotRef) {
    unsafe {
        let buf = seL4_GetIPCBuffer();
        (*buf).receiveCNode = slot.root.to_cap();
        (*buf).receiveIndex = slot.cptr;
        (*buf).receiveDepth = slot.depth as seL4_Word;
    }
}

/// Gets the current (thread-local) capability transfer destination.
pub fn get_cap_destination() -> SlotRef {
    unsafe {
        let buf = seL4_GetIPCBuffer();
        SlotRef::new(CNode::from_cap(
            (*buf).receiveCNode),
            (*buf).receiveIndex,
            (*buf).receiveDepth as u8,
        )
    }
}

/// Yield the remainder of the current timeslice back to the scheduler.
#[inline(always)]
pub fn yield_now() {
    unsafe {
        seL4_Yield();
    }
}

/// A handle for using core::fmt with seL4_DebugPutChar
pub struct DebugOutHandle;

impl ::core::fmt::Write for DebugOutHandle {
    fn write_str(&mut self, s: &str) -> ::core::fmt::Result {
        for &b in s.as_bytes() {
            unsafe { sel4_sys::seL4_DebugPutChar(b) };
        }
        Ok(())
    }
}
