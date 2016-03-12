// Copyright (c) 2015 The Robigalia Project Developers
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>,
// at your option. All files in the project carrying such
// notice may not be copied, modified, or distributed except
// according to those terms.

//! IRQ management and handling.

use sel4_sys::*;
use ToCap;

cap_wrapper_inner!{
    #[doc = "Authority to create IRQHandler capabilities"]
    :IRQControl
    #[doc = "Authority to wait for and acknowledge IRQs"]
    :IRQHandler
}

impl IRQControl {
    /// Create an IRQ handler capability, storing it in `slot`.
    pub fn get(&self, irq: isize, slot: ::SlotRef) -> ::Result {
        errcheck!(seL4_IRQControl_Get(self.cptr, irq, slot.root.to_cap(), slot.cptr, slot.depth));
    }
}

impl IRQHandler {
    /// Acknowledge receipt of this interrupt and re-enable it.
    ///
    /// If you don't ack interrupts, you'll never get them again.
    #[inline(always)]
    pub fn acknowledge(&self) -> ::Result {
        errcheck!(seL4_IRQHandler_Ack(self.cptr));
    }

    /// Set the mode of this IRQ.
    #[inline(always)]
    pub fn set_mode(&self, level_triggered: bool, low_polarity: bool) -> ::Result {
        let level_triggered = if level_triggered {
            1
        } else {
            0
        };
        let low_polarity = if low_polarity {
            1
        } else {
            0
        };
        errcheck!(seL4_IRQHandler_SetMode(self.cptr, level_triggered, low_polarity));
    }

    /// Set the notification object to notify when an interrupt is received.
    #[inline(always)]
    pub fn set_notification(&self, notification: ::Notification) -> ::Result {
        errcheck!(seL4_IRQHandler_SetNotification(self.cptr, notification.to_cap()));
    }

    /// Clear the notification object from this IRQ handler.
    #[inline(always)]
    pub fn clear_notification(&self) -> ::Result {
        errcheck!(seL4_IRQHandler_Clear(self.cptr));
    }
}
