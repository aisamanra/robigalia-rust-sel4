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

/// An unforgeable marker on a capability.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Badge {
    bits: seL4_CapData,
}

impl Badge {
    pub fn new(val: u32) -> Badge {
        let mut bits: seL4_CapData = unsafe { ::core::mem::zeroed() };
        bits.set_Badge(val as seL4_Word);
        Badge {
            bits: bits
        }
    }

    pub fn get_value(&self) -> u32 {
        self.bits.get_Badge() as u32
    }
}

/// A qualified reference to a capability slot.
///
/// This has three fields: a CPtr to a CNode, a CPtr, and a depth. Together, this information
/// specifies precisely how a slot is addressed, as far as the kernel is concerned.
///
/// This is used to specify slots in CNode methods.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct SlotRef {
    /// The CNode in our CSpace which acts as the "root" of this reference.
    ///
    /// The index and depth are interpreted relative to this CNode.
    pub root: CNode,
    /// A CPtr, relative to the `root`, to the desired slot.
    pub cptr: seL4_Word,
    /// Number of bits of `index` to strip off before arriving at the bits that the radix would be
    /// placed in.
    pub depth: u8,
}

impl SlotRef {
    /// Create a new slot reference from all component data
    #[inline(always)]
    pub fn new(root: CNode, cptr: seL4_Word, depth: u8) -> SlotRef {
        SlotRef {
            root: root,
            cptr: cptr,
            depth: depth,
        }
    }

    /// Copy the capability in this slot into `dest`, inheriting `rights`.
    #[inline(always)]
    pub fn copy(&self, dest: SlotRef, rights: seL4_CapRights) -> ::Result {
        errcheck!(seL4_CNode_Copy(dest.root.to_cap(),
                                  dest.cptr,
                                  dest.depth,
                                  self.root.to_cap(),
                                  self.cptr,
                                  self.depth,
                                  rights))
    }

    /// Remove the capability in this slot, replacing it with the null capability.
    #[inline(always)]
    pub fn delete(&self) -> ::Result {
        errcheck!(seL4_CNode_Delete(self.root.to_cap(), self.cptr, self.depth));
    }

    /// Copy the capability in this slot into `dest`, inheriting `rights` and applying `badge`.
    #[inline(always)]
    pub fn mint(&self, dest: SlotRef, rights: seL4_CapRights, badge: Badge) -> ::Result {
        errcheck!(seL4_CNode_Mint(dest.root.to_cap(),
                                  dest.cptr,
                                  dest.depth,
                                  self.root.to_cap(),
                                  self.cptr,
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
                                  dest.cptr,
                                  dest.depth,
                                  self.root.to_cap(),
                                  self.cptr,
                                  self.depth));
    }

    /// Move the capability in this slot into `dest`, applying `badge` and clearing this slot.
    #[inline(always)]
    pub fn mutate(&self, dest: SlotRef, badge: Badge) -> ::Result {
        errcheck!(seL4_CNode_Mutate(dest.root.to_cap(),
                                    dest.cptr,
                                    dest.depth,
                                    self.root.to_cap(),
                                    self.cptr,
                                    self.depth,
                                    badge.bits));
    }

    /// When used on a badged endpoint cap, cancel any outstanding send operations for that
    /// endpoint and badge.
    ///
    /// This has no effect on other objects.
    #[inline(always)]
    pub fn cancel_badged_sends(&self) -> ::Result {
        errcheck!(seL4_CNode_CancelBadgedSends(self.root.to_cap(), self.cptr, self.depth));
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
        errcheck!(seL4_CNode_Revoke(self.root.to_cap(), self.cptr, self.depth));
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
                                    dest.cptr,
                                    dest.depth,
                                    dest_badge.bits,
                                    pivot.root.to_cap(),
                                    pivot.cptr,
                                    pivot.depth,
                                    pivot_badge.bits,
                                    src.root.to_cap(),
                                    src.cptr,
                                    src.depth));
    }

    /// Save the reply capability into this slot.
    #[inline(always)]
    pub fn save_caller(&self) -> ::Result {
        errcheck!(seL4_CNode_SaveCaller(self.root.to_cap(), self.cptr, self.depth));
    }
}

/// Extra information needed to know how to address caps in a CNode.
///
/// This information isn't needed to interact with the kernel, but is necessary for reconstructing
/// SlotRefs given only CNode. The kernel already tracks this information for use during capability
/// lookup, but it is not possible to access the kernel's copy of this information.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct CNodeInfo {
    /// The value of the guard.
    ///
    /// This value should be "raw" and not shifted. Other code will shift this value as necessary
    /// to encode it in a cptr.
    pub guard_val: seL4_Word,
    /// Number of bits in the radix.
    ///
    /// The number of slots in this CNode is 2^radix_bits
    pub radix_bits: u8,
    /// Number of bits in the guard.
    pub guard_bits: u8,
    /// Number of bits before the guard.
    pub prefix_bits: u8,
}

/// Components of a CPtr unpacked.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct DecodedCPtr {
    pub prefix: seL4_Word,
    pub guard: seL4_Word,
    pub radix: seL4_Word,
    pub leftover: seL4_Word,
}

impl CNodeInfo {
    /// Decode a CPtr into 4 pieces: prefix, guard, radix, and leftover, like such:
    ///
    /// ```
    /// pppppppp|gggg|rrrrrrrr|llllllllllll
    /// ^-------|^---|^-------|^-----------
    /// |       |    |        |
    /// |       |    |        + leftover bits = 32 - 8 - 4 - 8 = 12
    /// |       |    + radix bits = 8
    /// |       + guard bits = 4
    /// + prefix bits = 8
    /// ```
    pub fn decode(&self, mut cptr: seL4_CPtr) -> DecodedCPtr {
        let mut decoded = DecodedCPtr {
            prefix: 0,
            guard: 0,
            radix: 0,
            leftover: 0
        };

        let leftover_bits = ::core::mem::size_of::<seL4_Word>()
            .wrapping_mul(8)
            .wrapping_sub(self.prefix_bits as usize)
            .wrapping_sub(self.guard_bits as usize)
            .wrapping_sub(self.prefix_bits as usize) as usize;
        let one = 1 as seL4_Word; // makes type inference work.

        decoded.leftover = cptr & one.wrapping_shl(leftover_bits as u32).wrapping_sub(1);
        cptr = cptr.wrapping_shr(leftover_bits as u32);

        decoded.radix = cptr & one.wrapping_shl(self.radix_bits as u32).wrapping_sub(1);
        cptr = cptr.wrapping_shr(self.radix_bits as u32);

        decoded.guard = cptr & one.wrapping_shl(self.guard_bits as u32).wrapping_sub(1);
        cptr = cptr.wrapping_shr(self.guard_bits as u32);

        decoded.prefix = cptr;

        decoded
    }

    pub fn encode(&self, decoded: &DecodedCPtr) -> seL4_CPtr {
        let leftover_bits = ::core::mem::size_of::<seL4_Word>()
            .wrapping_mul(8)
            .wrapping_sub(self.prefix_bits as usize)
            .wrapping_sub(self.guard_bits as usize)
            .wrapping_sub(self.prefix_bits as usize) as usize;
        let one = 1 as seL4_Word; // makes type inference work.
        let mut result = decoded.prefix &
            (one.wrapping_shl(self.prefix_bits as u32).wrapping_sub(1));
        result = result.wrapping_shl(self.guard_bits as u32);
        result |= decoded.guard & 
            (one.wrapping_shl(self.guard_bits as u32).wrapping_sub(1));
        result = result.wrapping_shl(self.radix_bits as u32);
        result |= decoded.radix & 
            (one.wrapping_shl(self.radix_bits as u32).wrapping_sub(1));
        result = result.wrapping_shl(self.prefix_bits as u32);
        result |= decoded.leftover & 
            (one.wrapping_shl(leftover_bits as u32).wrapping_sub(1));
        result
    }
}

/// A window into a CNode - a range of capability slots
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Window {
    /// Destination CNode to store the capabilities.
    pub cnode: SlotRef,
    /// Index into the CNode specified by `cnode` to start storing capabilities.
    ///
    /// That is, the first radix to start using.
    pub first_slot_idx: usize,
    /// Number of slots starting at first_slot_idx to use.
    pub num_slots: usize,
}

impl Window {
    /// Create a CPtr to the i'th cap in this window.
    pub fn cptr_to(&self, info: &CNodeInfo, i: usize) -> Option<seL4_CPtr> {
        if i >= self.num_slots {
            return None;
        }

        let raw = DecodedCPtr {
            prefix: self.cnode.cptr,
            guard: info.guard_val,
            radix: self.first_slot_idx.wrapping_add(i) as seL4_Word,
            leftover: 0,
        };

        Some(info.encode(&raw))
    }

    pub fn slotref_to(&self, info: &CNodeInfo, i: usize) -> Option<SlotRef> {
        match self.cptr_to(info, i) {
            Some(cptr) => {
                Some(SlotRef {
                    root: self.cnode.root,
                    cptr: cptr,
                    depth: info.radix_bits
                        .wrapping_add(info.guard_bits)
                        .wrapping_add(info.prefix_bits)
                })
            },
            None => None
        }
    }
}

cap_wrapper!{
    #[doc = "Fixed-length table for storing capabilities"]
    :CNode seL4_CapTableObject |i| 16*i
}
