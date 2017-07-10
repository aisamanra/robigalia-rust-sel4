// Copyright (c) 2015 The Robigalia Project Developers
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>,
// at your option. All files in the project carrying such
// notice may not be copied, modified, or distributed except
// according to those terms.

use sel4_sys::*;

use ToCap;

cap_wrapper!{ ()
    /// Authority to allocate ASID pools
    ASIDControl,
    /// Authority to create page directories
    ASIDPool,

    /// A 4K page of physical memory mapped into a page table
    SmallPage = seL4_ARM_SmallPageObject |_| 1 << 10,
    /// A 64K page of physical memory mapped into a page table
    LargePage = seL4_ARM_LargePageObject |_| 1 << 16,
    /// A 1M page of physical memory mapped into a page directory
    Section = seL4_ARM_SectionObject |_| 1 << 20,
    /// A 16M page of physical memory mapped into a page directory
    SuperSection = seL4_ARM_SuperSectionObject |_| 1 << 24,
    /// A page table, which can have pages mapped into it
    PageTable = seL4_ARM_PageTableObject |_| 1 << 10,
    /// A page directory, which holds page tables or sections and forms the root of the vspace
    PageDirectory = seL4_ARM_PageDirectoryObject |_| 1 << 14,
}

impl ASIDControl {
    /// Create a new ASID pool, using `untyped` as the storage, and storing the capability in
    /// `dest`.
    ///
    /// `untyped` must be 4KiB.
    #[inline(always)]
    pub fn make_pool(&self, untyped: SmallPage, dest: ::SlotRef) -> ::Result {
        unsafe_as_result!(seL4_ARM_ASIDControl_MakePool(
            self.cptr,
            untyped.to_cap(),
            dest.root.to_cap(),
            dest.cptr,
            dest.depth,
        ))
    }
}

impl ASIDPool {
    /// Assign a page directory to this ASID pool.
    #[inline(always)]
    pub fn assign(&self, vroot: PageDirectory) -> ::Result {
        unsafe_as_result!(seL4_ARM_ASIDPool_Assign(self.cptr, vroot.to_cap()))
    }
}

macro_rules! page_impls {
    ($name:ident) => {
impl $name {
    /// Map this page into an address space.
    #[inline(always)]
    pub fn map(&self, pd: PageDirectory, addr: seL4_Word, rights: seL4_CapRights,
               attr: seL4_ARM_VMAttributes) -> ::Result {
        unsafe_as_result!(seL4_ARM_Page_Map(self.cptr, pd.to_cap(), addr, rights, attr))
    }

    /// Remap this page, possibly changing rights or attribute but not address.
    #[inline(always)]
    pub fn remap(&self, pd: PageDirectory, rights: seL4_CapRights,
                 attr: seL4_ARM_VMAttributes) -> ::Result {
        unsafe_as_result!(seL4_ARM_Page_Remap(self.cptr, pd.to_cap(), rights, attr))
    }

    /// Unmap this page.
    #[inline(always)]
    pub fn unmap(&self) -> ::Result {
        unsafe_as_result!(seL4_ARM_Page_Unmap(self.cptr))
    }

    /// Get the physical address of the underlying frame.
    ///
    /// **!!NOTE!!**: This is not exposed by libsel4 and thus should be considered unstable.
    #[inline(always)]
    pub fn __get_address(&self) -> Result<seL4_Word, ::Error> {
        let res = unsafe { seL4_ARM_Page_GetAddress(self.cptr) };
        if res.error == 0 {
            Ok(res.paddr)
        } else {
            Err(::Error(::GoOn::CheckIPCBuf))
        }
    }
}
}}

page_impls!(SmallPage);
page_impls!(LargePage);
page_impls!(Section);
page_impls!(SuperSection);

impl PageTable {
    /// Map this page table into an address space.
    #[inline(always)]
    pub fn map(&self, pd: PageDirectory, addr: seL4_Word, attr: seL4_ARM_VMAttributes) -> ::Result {
        unsafe_as_result!(seL4_ARM_PageTable_Map(self.cptr, pd.to_cap(), addr, attr))
    }

    /// Unmap this page.
    #[inline(always)]
    pub fn unmap(&self) -> ::Result {
        unsafe_as_result!(seL4_ARM_PageTable_Unmap(self.cptr))
    }
}
