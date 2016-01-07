// Copyright (c) 2015 The Robigalia Project Developers
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>,
// at your option. All files in the project carrying such
// notice may not be copied, modified, or distributed except
// according to those terms.

//! Dealing with CNodes

use sel4_sys::*;
use ToCap;

/// A qualified reference to a capability slot.
///
/// This has three fields: a CPtr to a CNode, a CPtr, and a depth. Together, this information
/// specifies precisely how a slot is addressed.
///
/// This is used to specify slots in CNode methods.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct SlotRef {
    /// The CNode which acts as the "root" of this reference.
    ///
    /// The index and depth are interpreted relative to this CNode.
    pub root: CNode,
    /// A CPtr, relative to the `root`, to the desired capability slot.
    pub index: seL4_Word,
    /// Number of bits of `index` to resolve before stopping resolution.
    pub depth: u8,
}

/// A window into a CNode - a range of capability slots
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Window {
    /// Destination CNode to store the capabilities.
    pub cnode: SlotRef,
    /// Index into the capability slot table to start storing capabilities.
    pub first_slot_idx: usize,
    /// Number of slots starting at first_slot_idx to use.
    pub num_slots: usize,
}

/// An unforgeable marker on a capability
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Badge {
    bits: seL4_CapData,
}

impl Badge {
    pub fn new(val: u32) -> Badge {
        let mut bits: seL4_CapData = unsafe { ::core::mem::zeroed() };
        bits.set_Badge(val);
        Badge {
            bits: bits
        }
    }

    pub fn get_value(&self) -> u32 {
        self.bits.get_Badge()
    }
}

impl SlotRef {
    /// Create a new slot reference from all component data
    #[inline(always)]
    pub fn new(root: CNode, index: seL4_Word, depth: u8) -> SlotRef {
        SlotRef {
            root: root,
            index: index,
            depth: depth,
        }
    }

    /// Copy the capability in this slot into `dest`, inheriting `rights`.
    #[inline(always)]
    pub fn copy(&self, dest: SlotRef, rights: seL4_CapRights) -> ::Result {
        errcheck!(seL4_CNode_Copy(dest.root.to_cap(),
                                  dest.index,
                                  dest.depth,
                                  self.root.to_cap(),
                                  self.index,
                                  self.depth,
                                  rights))
    }

    /// Remove the capability in this slot, replacing it with the null capability.
    #[inline(always)]
    pub fn delete(&self) -> ::Result {
        errcheck!(seL4_CNode_Delete(self.root.to_cap(), self.index, self.depth));
    }

    /// Copy the capability in this slot into `dest`, inheriting `rights` and applying `badge`.
    #[inline(always)]
    pub fn mint(&self, dest: SlotRef, rights: seL4_CapRights, badge: Badge) -> ::Result {
        errcheck!(seL4_CNode_Mint(dest.root.to_cap(),
                                  dest.index,
                                  dest.depth,
                                  self.root.to_cap(),
                                  self.index,
                                  self.depth,
                                  rights,
                                  badge.bits));
    }

    /// Move the capability in this slot into `dest`, clearing this slot.
    ///
    /// Note: This is called `move_` because `move` is a keyword in Rust.
    #[inline(always)]
    pub fn move_(&self, dest: SlotRef) -> ::Result {
        errcheck!(seL4_CNode_Move(dest.root.to_cap(),
                                  dest.index,
                                  dest.depth,
                                  self.root.to_cap(),
                                  self.index,
                                  self.depth));
    }

    /// Move the capability in this slot into `dest`, applying `badge` and clearing this slot.
    #[inline(always)]
    pub fn mutate(&self, dest: SlotRef, badge: Badge) -> ::Result {
        errcheck!(seL4_CNode_Mutate(dest.root.to_cap(),
                                    dest.index,
                                    dest.depth,
                                    self.root.to_cap(),
                                    self.index,
                                    self.depth,
                                    badge.bits));
    }

    /// "Recycle" the capability in this slot.
    ///
    /// It's not clear to me what this does, precisely, since I haven't consulted the spec or
    /// thought much about it, but it's advertised as for "reusing an object within the same
    /// protection domain".
    #[inline(always)]
    pub fn recycle(&self) -> ::Result {
        errcheck!(seL4_CNode_Recycle(self.root.to_cap(), self.index, self.depth));
    }

    /// Delete all child capabilities of the capability in this slot.
    ///
    /// Do note the two nasty cases in the manual:
    ///
    /// - If the last cap to the TCB for the currently running thread is deleted, the thread will
    /// be destroyed at that point and further child capabilities will not be deleted
    /// - If the last cap to the memory storing this CNode is deleted, something bad happens and
    /// the revoke will stop.
    #[inline(always)]
    pub fn revoke(&self) -> ::Result {
        errcheck!(seL4_CNode_Revoke(self.root.to_cap(), self.index, self.depth));
    }

    /// Atomically "rotate" the capability in `second` into `destination` applying
    /// `destination_badge`, and the capability in `src` into `pivot` applying `pivot_badge`.
    ///
    /// This is an associated function instead of a method because it's not really clear which slot
    /// deserves to be the receiver.
    #[inline(always)]
    pub fn rotate(destination: SlotRef,
                  destination_badge: Badge,
                  pivot: SlotRef,
                  pivot_badge: Badge,
                  src: SlotRef)
                  -> ::Result {
        let dest = destination;
        let dest_badge = destination_badge;
        errcheck!(seL4_CNode_Rotate(dest.root.to_cap(),
                                    dest.index,
                                    dest.depth,
                                    dest_badge.bits,
                                    pivot.root.to_cap(),
                                    pivot.index,
                                    pivot.depth,
                                    pivot_badge.bits,
                                    src.root.to_cap(),
                                    src.index,
                                    src.depth));
    }

    /// Save the reply capability into this slot.
    #[inline(always)]
    pub fn save_caller(&self) -> ::Result {
        errcheck!(seL4_CNode_SaveCaller(self.root.to_cap(), self.index, self.depth));
    }
}

cap_wrapper!{
    #[doc = "Fixed-length table for storing capabilities"]
    :CNode seL4_CapTableObject
}
