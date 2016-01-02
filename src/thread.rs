// Copyright (c) 2015 The Robigalia Project Developers
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>,
// at your option. All files in the project carrying such
// notice may not be copied, modified, or distributed except
// according to those terms.


use sel4_sys::*;

use {ToCap, Notification, CNode};

cap_wrapper!{
    #[doc="A thread control block"]
    :Thread
}

/// Thread configuration.
///
/// Set `Thread::set_space` and `Thread::set_ipc_buffer` for details about those portions of this
/// structure.
pub struct ThreadConfiguration {
    pub fault_handler: seL4_Word,
    pub priority: u8,
    pub cspace_root: CNode,
    pub cspace_root_data: seL4_CapData,
    pub vspace_root: seL4_CPtr,
    pub vspace_root_data: seL4_CapData,
    pub buffer: seL4_Word,
    pub buffer_frame: seL4_Word,
}

impl Thread {
    /// Bind a notification object to this thread.
    #[inline(always)]
    pub fn bind_notification(&self, notification: Notification) -> ::Result {
        errcheck!(seL4_TCB_BindNotification(self.cptr, notification.to_cap()));
    }

    /// Unbind any notification object from this thread.
    #[inline(always)]
    pub fn unbind_notification(&self) -> ::Result {
        errcheck!(seL4_TCB_UnbindNotification(self.cptr));
    }

    /// Configure this thread with new parameters.
    #[inline(always)]
    pub fn configure(&self, config: ThreadConfiguration) -> ::Result {
        errcheck!(seL4_TCB_Configure(self.cptr,
                                     config.fault_handler,
                                     config.priority,
                                     config.cspace_root.to_cap(),
                                     config.cspace_root_data,
                                     config.vspace_root.to_cap(),
                                     config.vspace_root_data,
                                     config.buffer,
                                     config.buffer_frame));
    }

    /// Copy the registers from this thread to `dest`.
    ///
    /// If `suspend_source` is true, this thread is suspended before the transfer.
    ///
    /// If `resume_dest` is true, the destination thread is resumed after the transfer.
    ///
    /// If `transfer_frame`, is true, frame registers will be transfered. These are the registers
    /// read, modified, or preserved by system calls.
    ///
    /// If `transfer_integer` is true, all the registers not transfered by `transfer_frame` will be
    /// transfered.
    #[inline(always)]
    pub fn copy_registers(&self,
                          dest: Thread,
                          suspend_source: bool,
                          resume_dest: bool,
                          transfer_frame: bool,
                          transfer_integer: bool,
                          arch_flags: u8)
                          -> ::Result {
        let suspend_source = if suspend_source {
            1
        } else {
            0
        };
        let resume_dest = if resume_dest {
            1
        } else {
            0
        };
        let transfer_frame = if transfer_frame {
            1
        } else {
            0
        };
        let transfer_integer = if transfer_integer {
            1
        } else {
            0
        };
        errcheck!(seL4_TCB_CopyRegisters(dest.cptr,
                                         self.cptr,
                                         suspend_source,
                                         resume_dest,
                                         transfer_frame,
                                         transfer_integer,
                                         arch_flags));
    }

    /// Read this thread's registers.
    ///
    /// If `suspend`, suspend this thread before copying.
    #[inline(always)]
    pub fn read_registers(&self,
                          suspend: bool,
                          arch_flags: u8)
                          -> Result<seL4_UserContext, ::Error> {
        let suspend = if suspend {
            1
        } else {
            0
        };
        let mut regs = unsafe { ::core::mem::zeroed() };
        let res = unsafe {
            seL4_TCB_ReadRegisters(self.cptr,
                                   suspend,
                                   arch_flags,
                                   (::core::mem::size_of::<seL4_UserContext>() / ::core::mem::size_of::<usize>()) as seL4_Word,
                                   &mut regs)
        };
        if res == 0 {
            Ok(regs)
        } else {
            Err(::Error(::GoOn::CheckIPCBuf))
        }
    }

    /// Resume this thread
    #[inline(always)]
    pub fn resume(&self) -> ::Result {
        errcheck!(seL4_TCB_Resume(self.cptr));
    }

    /// Set this thread's IPC buffer.
    ///
    /// `address` is where in the virtual address space the IPC buffer will be located, and `frame`
    /// is a capability to the physical memory that will back that page. `address` must be
    /// naturally aligned to 512-bytes.
    #[inline(always)]
    pub fn set_ipc_buffer(&self, address: seL4_Word, frame: seL4_CPtr) -> ::Result {
        errcheck!(seL4_TCB_SetIPCBuffer(self.cptr, address, frame));
    }

    /// Set this thread's priority.
    ///
    /// This can only set the priority to lower or equal to the priority of the thread that makes
    /// this request.
    #[inline(always)]
    pub fn set_priority(&self, priority: u8) -> ::Result {
        errcheck!(seL4_TCB_SetPriority(self.cptr, priority));
    }

    /// Set this thread's fault endpoint, CSpace, and VSpace.
    ///
    /// The fault endpoint is a CPtr interpreted in the new CSpace.
    ///
    /// The CSpace root data is the new guard and guard size of the new root CNode, though if it's
    /// zero it is ignored.
    ///
    /// The VSpace root data is ignored on x86 and ARM.
    #[inline(always)]
    pub fn set_space(&self,
                     fault_endpoint: seL4_CPtr,
                     cspace_root: CNode,
                     cspace_root_data: seL4_CapData,
                     vspace_root: seL4_CPtr,
                     vspace_root_data: seL4_CapData)
                     -> ::Result {
        errcheck!(seL4_TCB_SetSpace(self.cptr,
                                    fault_endpoint,
                                    cspace_root.to_cap(),
                                    cspace_root_data,
                                    vspace_root,
                                    vspace_root_data));
    }

    /// Suspend this thread.
    #[inline(always)]
    pub fn suspend(&self) -> ::Result {
        errcheck!(seL4_TCB_Suspend(self.cptr));
    }

    /// Set this thread's registers from the provided context.
    ///
    /// If `resume`, resume this thread after writing.
    #[inline(always)]
    pub fn write_registers(&self,
                           resume: bool,
                           arch_flags: u8,
                           regs: &seL4_UserContext)
                           -> ::Result {
        let resume = if resume {
            1
        } else {
            0
        };
        let res = unsafe {
            seL4_TCB_WriteRegisters(self.cptr, resume, arch_flags,
                                    (::core::mem::size_of::<seL4_UserContext>() / ::core::mem::size_of::<usize>()) as seL4_Word,
                                    regs as *const seL4_UserContext as *mut _)
        };
        if res == 0 {
            Ok(())
        } else {
            Err(::Error(::GoOn::CheckIPCBuf))
        }
    }

    /// Set this thread's domain.
    #[inline(always)]
    pub fn set_domain(&self, domain: u8, domain_control: ::DomainSet) -> ::Result {
        errcheck!(seL4_DomainSet_Set(domain_control.to_cap(), domain, self.cptr));
    }
}
