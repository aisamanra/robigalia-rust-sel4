// Copyright (c) 2015 The Robigalia Project Developers
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>,
// at your option. All files in the project carrying such
// notice may not be copied, modified, or distributed except
// according to those terms.

use sel4_sys::*;

cap_wrapper!{
    #[doc="A notification object for signalling"]
    :Notification
}

impl Notification {
    /// Signal the notification.
    #[inline(always)]
    pub fn signal(&self) {
        unsafe { seL4_Signal(self.cptr) }
    }

    /// Block waiting for the notification to be signaled.
    ///
    /// Returns the notification word.
    #[inline(always)]
    pub fn wait(&self) -> seL4_Word {
        let mut ret = 0;
        unsafe {
            seL4_Recv(self.cptr, &mut ret);
        }
        ret
    }

    /// Poll the notification.
    ///
    /// The notification word is cleared, and the old notification word is returned.
    #[inline(always)]
    pub fn poll(&self) -> seL4_Word {
        let mut ret = 0;
        unsafe {
            seL4_NBRecv(self.cptr, &mut ret);
        }
        ret
    }
}
