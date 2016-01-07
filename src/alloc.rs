// Copyright (c) 2015 The Robigalia Project Developers
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>,
// at your option. All files in the project carrying such
// notice may not be copied, modified, or distributed except
// according to those terms.

//! Traits for basic object allocation and cspace management.

use sel4_sys::*;
use {CNode, Allocatable, SlotRef, Error};

/// Interface for allocating objects.
pub trait ObjectAllocator {
    /// Allocate a slot in the CSpace rooted at `relative_to` if it is `Some`. Otherwise, allocate
    /// a slot in this thread's CSpace.
    fn allocate_slot(&self, relative_to: Option<CNode>) -> Option<seL4_CPtr>;
    /// Mark a slot unused and available for allocation.
    fn free_slot(&self, cptr: seL4_CPtr);

    /// Return the full-qualified slot reference for a CPtr, for use in `allocate_object`.
    ///
    /// Note: this is required because the object allocator knows what the cspace layout looks
    /// like, and other code might not.
    fn relativize(&self, cptr: seL4_CPtr) -> Option<SlotRef>;

    /// Allocate an object, storing the capability into the specified slot.
    fn allocate_object<T: Allocatable>(&self, dest: SlotRef) -> Result<Option<T>, Error>;
    /// Free an object, deleting it (thus removing it from the capability derivation tree) and
    /// return the memory for use by the allocator.
    fn free_object<T: Allocatable>(&self, obj: T) -> Result<(), Error>;
}
