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

cap_wrapper!{
    :ASIDControl
    :ASIDPool
    :IOPort
    :IOSpace
    :IOPageTable
    :Page
    :PageTable
    :PageDirectory
}

impl ASIDControl {
    /// Create a new ASID pool, using `untyped` as the storage, and storing the capability in
    /// `dest`.
    ///
    /// `untyped` must be 4KiB.
    #[inline(always)]
    pub fn make_pool(&self, untyped: seL4_CPtr, dest: ::SlotRef) -> ::Result {
        errcheck!(seL4_IA32_ASIDControl_MakePool(self.cptr, untyped, dest.root.to_cap(), dest.index, dest.depth));
    }
}

impl ASIDPool {
    /// Assign a page directory to this ASID pool.
    #[inline(always)]
    pub fn assign(&self, vroot: PageDirectory) -> ::Result {
        errcheck!(seL4_IA32_ASIDPool_Assign(self.cptr, vroot.to_cap()));
    }
}

impl IOPort {
    /// Read 8 bits from the given port.
    #[inline(always)]
    pub fn read8(&self, port: u16) -> Result<u8, ::Error> {
        let res = unsafe { seL4_IA32_IOPort_In8(self.cptr, port) };
        if res.error == 0 {
            Ok(res.result)
        } else {
            Err(::Error(::GoOn::CheckIPCBuf))
        }
    }

    /// Read 16 bits from the given port.
    #[inline(always)]
    pub fn read16(&self, port: u16) -> Result<u16, ::Error> {
        let res = unsafe { seL4_IA32_IOPort_In16(self.cptr, port) };
        if res.error == 0 {
            Ok(res.result)
        } else {
            Err(::Error(::GoOn::CheckIPCBuf))
        }
    }

    /// Read 32 bits from the given port.
    #[inline(always)]
    pub fn read32(&self, port: u16) -> Result<u32, ::Error> {
        let res = unsafe { seL4_IA32_IOPort_In32(self.cptr, port) };
        if res.error == 0 {
            Ok(res.result)
        } else {
            Err(::Error(::GoOn::CheckIPCBuf))
        }
    }

    /// Write 8-bit `value` to the given port.
    #[inline(always)]
    pub fn write8(&self, port: u16, value: u8) -> ::Result {
        errcheck!(seL4_IA32_IOPort_Out8(self.cptr, port, value));
    }

    /// Write 16-bit `value` to the given port.
    #[inline(always)]
    pub fn write16(&self, port: u16, value: u16) -> ::Result {
        errcheck!(seL4_IA32_IOPort_Out16(self.cptr, port, value));
    }

    /// Write 32-bit `value` to the given port.
    #[inline(always)]
    pub fn write32(&self, port: u16, value: u32) -> ::Result {
        errcheck!(seL4_IA32_IOPort_Out32(self.cptr, port, value));
    }
}

impl IOPageTable {
    /// Map this page table into an IOSpace at `addr`
    #[inline(always)]
    pub fn map(&self, iospace: IOSpace, addr: seL4_Word) -> ::Result {
        errcheck!(seL4_IA32_IOPageTable_Map(self.cptr, iospace.to_cap(), addr));
    }
}


impl Page {
    /// Map this page into an IOSpace with `rights` at `addr`.
    #[inline(always)]
    pub fn map_io(&self, iospace: IOSpace, rights: seL4_CapRights, addr: seL4_Word) -> ::Result {
        errcheck!(seL4_IA32_Page_MapIO(self.cptr, iospace.to_cap(), rights, addr));
    }

    /// Map this page into an address space.
    #[inline(always)]
    pub fn map(&self, pd: PageDirectory, addr: seL4_Word, rights: seL4_CapRights, attr: seL4_IA32_VMAttributes) -> ::Result {
        errcheck!(seL4_IA32_Page_Map(self.cptr, pd.to_cap(), addr, rights, attr));
    }

    /// Remap this page, possibly changing rights or attribute but not address.
    #[inline(always)]
    pub fn remap(&self, pd: PageDirectory, rights: seL4_CapRights, attr: seL4_IA32_VMAttributes) -> ::Result {
        errcheck!(seL4_IA32_Page_Remap(self.cptr, pd.to_cap(), rights, attr));
    }

    /// Unmap this page.
    #[inline(always)]
    pub fn unmap(&self) -> ::Result {
        errcheck!(seL4_IA32_Page_Unmap(self.cptr));
    }

    /// Get the physical address of the underlying frame.
    ///
    /// **!!NOTE!!**: This is not exposed by libsel4 and thus should be considered unstable.
    #[inline(always)]
    pub fn __get_address(&self) -> Result<seL4_Word, ::Error> {
        let res = unsafe { seL4_IA32_Page_GetAddress(self.cptr) };
        if res.error == 0 {
            Ok(res.paddr)
        } else {
            Err(::Error(::GoOn::CheckIPCBuf))
        }
    }
}

impl PageTable {
    /// Map this page table into an address space.
    #[inline(always)]
    pub fn map(&self, pd: PageDirectory, addr: seL4_Word, attr: seL4_IA32_VMAttributes) -> ::Result {
        errcheck!(seL4_IA32_PageTable_Map(self.cptr, pd.to_cap(), addr, attr));
    }

    /// Unmap this page.
    #[inline(always)]
    pub fn unmap(&self) -> ::Result {
        errcheck!(seL4_IA32_PageTable_Unmap(self.cptr));
    }
}
