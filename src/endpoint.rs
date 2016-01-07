// Copyright (c) 2015 The Robigalia Project Developers
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>,
// at your option. All files in the project carrying such
// notice may not be copied, modified, or distributed except
// according to those terms.

//! Using endpoints for message passing
//!
//! In seL4, message passing is the fundamental primitive upon which the entire system is built.
//! All kernel services are accessed through messages sent to capabilities which the kernel
//! recognizes as belonging to kernel objects. Threads can also use this mechanism to send messages
//! between themselves.
//!
//! Endpoints represent authorization to receive or send messages for a particular queue. These
//! queues do not have a buffer, and act as a rendezvous. That is, senders block until there is a
//! receiver and receivers block until there is a sender - at which point they meet, the message is
//! copied directly from the source to its final destination, and the threads continue execution.
//! Multiple threads can be waiting to send or receive on the same queue. A message is delivered
//! from exactly one sender to exactly one receiver.
//!
//! In addition to being able to send data, capabilities can also be transfered between threads.
//! The endpoint must have the `CanGrant` bit set in its rights. In practice, only one new
//! capability can be transfered at a time - the actual situation is somewhat more complex. Refer
//! to §4.2.2 ("Capability Transfer") of the seL4 Reference Manual. The slot where the received
//! capability will be stored is global state not tied to any particular endpoint.
//!
//! Do note that `Endpoint` does not also attempt to model notification objects, instead leaving
//! that to the `Notification` type.

use sel4_sys::*;

cap_wrapper!{
    #[doc="An endpoint for message passing"]
    :Endpoint seL4_EndpointObject
}

/// The result of a successful receive.
///
/// Contains "sender information", which is the badge of the endpoint which was invoked to send a
/// message.
///
/// Also contains the decoded message information.
pub struct RecvToken {
    pub badge: seL4_Word,
    pub label: seL4_Word,
    caps_unwrapped: seL4_Word,
    len: seL4_Word,
}

impl RecvToken {
    fn from_raw(sender: seL4_Word, message_info: seL4_MessageInfo) -> RecvToken {
        RecvToken {
            badge: sender,
            label: message_info.get_label(),
            caps_unwrapped: message_info.get_capsUnwrapped(),
            len: message_info.get_length(),
        }
    }

    /// Read out unwrapped capabilities into a slice.
    ///
    /// Returns `Err` if the slice is not at least length `caps_unwrapped`.
    pub fn get_unwrapped_caps(&self, caps: &mut [seL4_Word]) -> Result<(), ()> {
        if caps.len() < seL4_MsgMaxExtraCaps && (caps.len() as seL4_Word) < self.caps_unwrapped {
            return Err(());
        }

        unsafe {
            ::core::intrinsics::copy_nonoverlapping(&(*seL4_GetIPCBuffer()).caps_or_badges as *const seL4_Word,
                                             caps.as_mut_ptr(), self.caps_unwrapped as usize)
        }

        Ok(())
    }

    /// Read out message data into a slice.
    ///
    /// Returns `Err` if the slice is not at least length `words_transferred`.
    pub fn get_data(&self, data: &mut [seL4_Word]) -> Result<(), ()> {
        if data.len() < seL4_MsgMaxLength && (data.len() as seL4_Word) < self.len {
            return Err(());
        }

        unsafe {
            ::core::intrinsics::copy_nonoverlapping(&(*seL4_GetIPCBuffer()).msg as *const seL4_Word,
                                                    data.as_mut_ptr(),
                                                    self.len as usize)
        }

        Ok(())

    }

    pub fn caps_unwrapped(&self) -> seL4_Word {
        self.caps_unwrapped
    }

    pub fn words_transferred(&self) -> seL4_Word {
        self.len
    }
}

impl Endpoint {
    /// Send data.
    #[inline(always)]
    pub fn send_data(&self, data: &[seL4_Word]) -> ::Result {
        self.send_message(data, &[])
    }

    /// Send a capability.
    #[inline(always)]
    pub fn send_cap<T: ::ToCap>(&self, cap: T) -> ::Result {
        self.send_message(&[], &[cap.to_cap()])
    }

    /// Send a message.
    ///
    /// The only failures that can occur are if `data` or `caps` is too long to fit in the IPC
    /// buffer. In this case, `TooMuchData` or `TooManyCaps` will be the error details,
    /// respectively.
    ///
    /// This is `seL4_Send` in its full generality.
    #[inline(always)]
    pub fn send_message(&self, data: &[u32], caps: &[seL4_CPtr]) -> ::Result {
        if data.len() > seL4_MsgMaxLength {
            return Err(::Error(::GoOn::TooMuchData));
        }
        if caps.len() > seL4_MsgMaxExtraCaps {
            return Err(::Error(::GoOn::TooManyCaps));
        }
        unsafe {
            let buf = seL4_GetIPCBuffer();
            ::core::ptr::copy_nonoverlapping(data.as_ptr(),
                                             (&mut (*buf).msg).as_mut_ptr(),
                                             data.len());
            ::core::ptr::copy_nonoverlapping(caps.as_ptr(),
                                             (&mut (*buf).caps_or_badges).as_mut_ptr(),
                                             caps.len());
            seL4_Send(self.cptr,
                      seL4_MessageInfo::new(0,
                                            0,
                                            caps.len() as seL4_Word,
                                            data.len() as seL4_Word));
            if (*buf).tag.get_label() != 0 {
                return Err(::Error(::GoOn::CheckIPCBuf));
            }
        }
        Ok(())
    }

    /// Try to send a message, returning no indication of failure if the message could not be sent.
    pub fn try_send_message(&self, data: &[u32], caps: &[seL4_CPtr]) -> ::Result {
        if data.len() > seL4_MsgMaxLength {
            return Err(::Error(::GoOn::TooMuchData));
        }
        if caps.len() > seL4_MsgMaxExtraCaps {
            return Err(::Error(::GoOn::TooManyCaps));
        }
        unsafe {
            let buf = seL4_GetIPCBuffer();
            ::core::ptr::copy_nonoverlapping(data.as_ptr(),
                                             (&mut (*buf).msg).as_mut_ptr(),
                                             data.len());
            ::core::ptr::copy_nonoverlapping(caps.as_ptr(),
                                             (&mut (*buf).caps_or_badges).as_mut_ptr(),
                                             caps.len());
            seL4_NBSend(self.cptr,
                        seL4_MessageInfo::new(0,
                                              0,
                                              caps.len() as seL4_Word,
                                              data.len() as seL4_Word));
            if (*buf).tag.get_label() != 0 {
                return Err(::Error(::GoOn::CheckIPCBuf));
            }
        }
        Ok(())
    }

    /// Block until a message is received.
    #[inline(always)]
    pub fn recv(&self) -> RecvToken {
        let mut sender = 0;
        let msginfo = unsafe { seL4_Recv(self.cptr, &mut sender) };
        RecvToken::from_raw(sender, msginfo)
    }

    /// Try to receive a message.
    ///
    /// If there is no message immediately available in the queue, the badge in `RecvToken` will be
    /// `0`. This is the only way to determine if a message was available.
    #[inline(always)]
    pub fn try_recv(&self) -> RecvToken {
        let mut sender = 0;
        let msginfo = unsafe { seL4_NBRecv(self.cptr, &mut sender) };
        RecvToken::from_raw(sender, msginfo)
    }
}
