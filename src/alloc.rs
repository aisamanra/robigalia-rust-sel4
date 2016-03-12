// Copyright (c) 2015 The Robigalia Project Developers
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>,
// at your option. All files in the project carrying such
// notice may not be copied, modified, or distributed except
// according to those terms.

//! Traits for basic object allocation and cspace management.

use {Allocatable, SlotRef};

/// Interface for allocating objects.
pub trait ObjectAllocator {
    type ObjectAllocError;
    type SlotFreeError;
    type ObjectFreeError;

    /// Otherwise, allocate a slot in this thread's CSpace.
    fn allocate_slot(&self) -> Option<SlotRef>;
    /// Mark a slot unused and available for allocation.
    fn free_slot(&self, slot: SlotRef) -> Result<(), Self::SlotFreeError>;

    /// Allocate an object, storing the capability into the specified slot.
    fn allocate_object<T: Allocatable>(&self, dest: SlotRef) -> Result<Option<T>, Self::ObjectAllocError>;
    /// Free an object, deleting it (thus removing it from the capability derivation tree) and
    /// return the memory for use by the allocator.
    fn free_object<T: Allocatable>(&self, obj: T) -> Result<(), Self::ObjectFreeError>;
}
