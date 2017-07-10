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
    /// Authority to create ASID pools
    ASIDControl,
    /// Authority to create page directories
    ASIDPool,
    /// Authority to use port-IO
    IOPort,
    /// Authority to map IO page tables into a device's address space
    IOSpace,

    /// A page directory pointer table, which holds page directories
    PDPT = seL4_X86_PDPTObject |_| 1 << seL4_PDPTBits,
    /// A page map level 4, which holds PDPTs
    PML4 = seL4_X64_PML4Object |_| 1 << seL4_PML4Bits,
    /// A huge (1G) page of physical memory that can be mapped into a vspace
    HugePage = seL4_X86_4K |_| 1 << seL4_HugePageBits,
    /// A (4K) page of physical memory that can be mapped into a vspace
    Page = seL4_X86_4K |_| 1 << seL4_PageBits,
    /// A large (2M) page of physical memory that can be mapped into a vspace
    LargePage = seL4_X86_LargePageObject |_| 1 << seL4_LargePageBits,
    /// A page table, which can have pages mapped into it
    PageTable = seL4_X86_PageTableObject |_| 1 << seL4_PageTableBits,
    /// A page directory, which holds page tables
    PageDirectory = seL4_X86_PageDirectoryObject |_| 1 << seL4_PageDirBits,
    /// A page table for the IOMMU
    IOPageTable = seL4_X86_IOPageTableObject |_| 1 << seL4_IOPageTableBits,
    /// A virtual CPU, for virtualization
    VCPU = seL4_X86_VCPUObject |_| 1 << seL4_VCPUBits,
    /// Extended page table (virt) PML4
    EPTPML4 = seL4_X86_EPTPML4Object |_| 1 << seL4_EPTPML4Bits,
    /// Extended page table (virt) PDPT
    EPTPDPT = seL4_X86_EPTPDPTObject |_| 1 << seL4_EPTPDPTBits,
    /// Extended page table (virt) PageDirectory
    EPTPageDirectory = seL4_X86_EPTPDObject |_| 1 << seL4_EPTPDBits,
    /// Extended page table (virt) PageTable
    EPTPageTable = seL4_X86_EPTPTObject |_| 1 << seL4_EPTPTBits,
}

impl ASIDControl {
    /// Create a new ASID pool, using `untyped` as the storage, and storing the capability in
    /// `dest`.
    ///
    /// `untyped` must be 4KiB.
    #[inline(always)]
    pub fn make_pool(&self, untyped: Page, dest: ::SlotRef) -> ::Result {
        unsafe_as_result!(seL4_X86_ASIDControl_MakePool(
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
        unsafe_as_result!(seL4_X86_ASIDPool_Assign(self.cptr, vroot.to_cap()))
    }
}

impl IOPort {
    /// Read 8 bits from the given port.
    #[inline(always)]
    pub fn read8(&self, port: u16) -> Result<u8, ::Error> {
        let res = unsafe { seL4_X86_IOPort_In8(self.cptr, port) };
        if res.error == 0 {
            Ok(res.result)
        } else {
            Err(::Error(::GoOn::CheckIPCBuf))
        }
    }

    /// Read 16 bits from the given port.
    #[inline(always)]
    pub fn read16(&self, port: u16) -> Result<u16, ::Error> {
        let res = unsafe { seL4_X86_IOPort_In16(self.cptr, port) };
        if res.error == 0 {
            Ok(res.result)
        } else {
            Err(::Error(::GoOn::CheckIPCBuf))
        }
    }

    /// Read 32 bits from the given port.
    #[inline(always)]
    pub fn read32(&self, port: u16) -> Result<u32, ::Error> {
        let res = unsafe { seL4_X86_IOPort_In32(self.cptr, port) };
        if res.error == 0 {
            Ok(res.result)
        } else {
            Err(::Error(::GoOn::CheckIPCBuf))
        }
    }

    /// Write 8-bit `value` to the given port.
    #[inline(always)]
    pub fn write8(&self, port: u16, value: u8) -> ::Result {
        unsafe_as_result!(seL4_X86_IOPort_Out8(self.cptr, port as seL4_Word, value as seL4_Word))
    }

    /// Write 16-bit `value` to the given port.
    #[inline(always)]
    pub fn write16(&self, port: u16, value: u16) -> ::Result {
        unsafe_as_result!(seL4_X86_IOPort_Out16(self.cptr, port as seL4_Word, value as seL4_Word))
    }

    /// Write 32-bit `value` to the given port.
    #[inline(always)]
    pub fn write32(&self, port: u16, value: u32) -> ::Result {
        unsafe_as_result!(seL4_X86_IOPort_Out32(self.cptr, port as seL4_Word, value as seL4_Word))
    }
}

impl IOPageTable {
    /// Map this page table into an IOSpace at `addr`
    #[inline(always)]
    pub fn map(&self, iospace: IOSpace, addr: seL4_Word) -> ::Result {
        unsafe_as_result!(seL4_X86_IOPageTable_Map(self.cptr, iospace.to_cap(), addr))
    }
}


impl Page {
    /// Map this page into an IOSpace with `rights` at `addr`.
    #[inline(always)]
    pub fn map_io(&self, iospace: IOSpace, rights: seL4_CapRights, addr: seL4_Word) -> ::Result {
        unsafe_as_result!(seL4_X86_Page_MapIO(self.cptr, iospace.to_cap(), rights, addr))
    }

    /// Map this page into an address space.
    #[inline(always)]
    pub fn map(&self, pd: PageDirectory, addr: seL4_Word, rights: seL4_CapRights,
               attr: seL4_X86_VMAttributes) -> ::Result {
        unsafe_as_result!(seL4_X86_Page_Map(self.cptr, pd.to_cap(), addr, rights, attr))
    }

    /// Remap this page, possibly changing rights or attribute but not address.
    #[inline(always)]
    pub fn remap(&self, pd: PageDirectory, rights: seL4_CapRights, attr: seL4_X86_VMAttributes)
                 -> ::Result {
        unsafe_as_result!(seL4_X86_Page_Remap(self.cptr, pd.to_cap(), rights, attr))
    }

    /// Unmap this page.
    #[inline(always)]
    pub fn unmap(&self) -> ::Result {
        unsafe_as_result!(seL4_X86_Page_Unmap(self.cptr))
    }

    /// Get the physical address of the underlying frame.
    #[inline(always)]
    pub fn get_address(&self) -> Result<seL4_Word, ::Error> {
        let res = unsafe { seL4_X86_Page_GetAddress(self.cptr) };
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
    pub fn map(&self, pd: PageDirectory, addr: seL4_Word, attr: seL4_X86_VMAttributes) -> ::Result {
        unsafe_as_result!(seL4_X86_PageTable_Map(self.cptr, pd.to_cap(), addr, attr))
    }

    /// Unmap this page.
    #[inline(always)]
    pub fn unmap(&self) -> ::Result {
        unsafe_as_result!(seL4_X86_PageTable_Unmap(self.cptr))
    }
}

impl PageDirectory {
    /// Get the status bits for a page mapped into this address space.
    ///
    /// Returns (accessed, dirty).
    #[inline(always)]
    pub fn get_status(&self, vaddr: usize) -> Result<(bool, bool), ::Error> {
        let res = unsafe { seL4_X86_PageDirectory_GetStatusBits(self.cptr, vaddr) };
        if res.error == 0 {
            unsafe {
                let buf = seL4_GetIPCBuffer();
                let accessed = (*buf).msg[0];
                let dirty = (*buf).msg[1];
                Ok((accessed == 1, dirty == 1))
            }
        } else {
            Err(::Error(::GoOn::CheckIPCBuf))
        }
    }

    /// Map this page directory into a PDPT
    #[inline(always)]
    pub fn map(&self, pdpt: PDPT, addr: seL4_Word, attr: seL4_X86_VMAttributes) -> ::Result {
        unsafe_as_result!(seL4_X86_PageDirectory_Map(self.cptr, pdpt.to_cap(), addr, attr))
    }

    /// Unmap this page directory from the PDPT it is mapped into
    pub fn unmap(&self) -> ::Result {
        unsafe_as_result!(seL4_X86_PageDirectory_Unmap(self.cptr))
    }
}

impl PDPT {
    /// Map this PDPT into a PML4
    #[inline(always)]
    pub fn map(&self, pml4: PML4, addr: seL4_Word, attr: seL4_X86_VMAttributes) -> ::Result {
        unsafe_as_result!(seL4_X86_PDPT_Map(self.cptr, pml4.to_cap(), addr, attr))
    }

    /// Unmap this PDPT from the PML4 it is mapped into
    #[inline(always)]
    pub fn unmap(&self) -> ::Result {
        unsafe_as_result!(seL4_X86_PDPT_Unmap(self.cptr))
    }
}

impl ::irq::IRQControl {
    /// Create an IRQHandler capability for a message-signalled interrupt (MSI).
    ///
    /// `pci_*` indicate the address of the PCI function that will generate the handled interrupt.
    ///
    /// `handle` is the value programmed into the data portion of the MSI.
    ///
    /// `vector` is the CPU vector the interrupt will be delivered to.
    #[inline(always)]
    pub fn get_msi(&self, slotref: ::SlotRef, pci_bus: seL4_Word, pci_dev: seL4_Word,
                   pci_func: seL4_Word, handle: seL4_Word, vector: seL4_Word) -> ::Result {
        unsafe_as_result!(seL4_IRQControl_GetMSI(
            self.to_cap(),
            slotref.root.to_cap(),
            slotref.cptr,
            slotref.depth as seL4_Word,
            pci_bus,
            pci_dev,
            pci_func,
            handle,
            vector,
        ))
    }

    /// Create an IRQHandler capability for an interrupt from an IOAPIC.
    ///
    /// `ioapic` is the zero-based index of the IOAPIC the interrupt will be delivered from, in the
    /// same order as in the ACPI tables.
    ///
    /// `pin` is the IOAPIC pin that generates the interrupt.
    ///
    /// `level_triggered` and `active_low` should be set based on the relevant HW the interrupt is
    /// for.
    ///
    /// `vector` is the CPU vector the interrupt will be delivered on.
    #[inline(always)]
    pub fn get_ioapic(&self, slotref: ::SlotRef, ioapic: seL4_Word, pin: seL4_Word,
                      level_triggered: bool, active_low: bool, vector: seL4_Word)
                      -> ::Result {
        unsafe_as_result!(seL4_IRQControl_GetIOAPIC(
            self.to_cap(),
            slotref.root.to_cap(),
            slotref.cptr,
            slotref.depth as seL4_Word,
            ioapic,
            pin,
            level_triggered as usize,
            active_low as usize,
            vector,
        ))
    }
}
