// Copyright (c) 2015 The Robigalia Project Developers
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>,
// at your option. All files in the project carrying such
// notice may not be copied, modified, or distributed except
// according to those terms.

use sel4_sys::*;

use Error;
use GoOn;

/// Detailed information about an error extracted from the message registers.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ErrorDetails {
    InvalidArgument {
        which: seL4_Word,
    },
    InvalidCapability {
        which: seL4_Word,
    },
    IllegalOperation,
    RangeError {
        min: seL4_Word,
        max: seL4_Word,
    },
    AlignmentError,
    FailedLookup {
        failed_for_source: bool,
        lookup_kind: LookupFailureKind,
    },
    TruncatedMessage,
    DeleteFirst,
    RevokeFirst,
    NotEnoughMemory {
        bytes_available: seL4_Word,
    },
    TooMuchData,
    TooManyCaps,
//  WouldBlock,
}

impl ::core::fmt::Display for ErrorDetails {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        use self::ErrorDetails::*;

        match *self {
            InvalidArgument { which } =>
                write!(f, "argument {} was invalid", which),
            InvalidCapability { which } =>
                write!(f, "capability {} was invalid", which),
            IllegalOperation =>
                write!(f, "the operation was not permitted"),
            RangeError { min, max } =>
                write!(f, "value was out of range (min = {}, max = {})", min, max),
            AlignmentError =>
                write!(f, "value was not aligned"),
            FailedLookup { failed_for_source, lookup_kind } =>
                write!(f, "looking up {}capability failed: {}",
                       if failed_for_source { "source " } else { "" }, lookup_kind),
            TruncatedMessage =>
                write!(f, "not enough arguments were provided (this indicates a bug in rust-sel4)"),
            DeleteFirst =>
                write!(f, "a capability should have been deleted before attempting the operation"),
            RevokeFirst =>
                write!(f, "a capability should have been revoked before attempting the operation"),
            NotEnoughMemory { bytes_available } =>
                write!(f, "there were {} bytes available, which was not enough to allocate the\
                           object", bytes_available),
            TooMuchData =>
                write!(f, "tried to send more data than can fit in the IPC buffer"),
            TooManyCaps =>
                write!(f, "tried to send more capabilities than can fit in the IPC buffer")
//          WouldBlock =>
//             write!(f, "tried to perform an operation on an endpoint that would have blocked"),
        }
    }
}

impl Error {
    #[inline]
    pub fn details(&self) -> Option<ErrorDetails> {
        use self::ErrorDetails::*;

        match self.0 {
            GoOn::TooMuchData => Some(TooMuchData),
            GoOn::TooManyCaps => Some(TooManyCaps),
//          GoOn::WouldBlock => Some(WouldBlock),
            GoOn::CheckIPCBuf => {
                unsafe {
                    let ipcbuf = seL4_GetIPCBuffer();
                    let label = (*ipcbuf).tag.get_label();
                    assert!(label > seL4_NotEnoughMemory as seL4_Word, "Unknown error type");

                    // unsafe: transmute could be replaced with an enum_from_primitive!()
                    match ::core::mem::transmute::<_, seL4_Error>(label) {
                         seL4_NoError => None,
                         seL4_InvalidArgument => Some(InvalidArgument { which: (*ipcbuf).msg[0] }),
                         seL4_InvalidCapability => Some(InvalidCapability { which: (*ipcbuf).msg[0] }),
                         seL4_IllegalOperation => Some(IllegalOperation),
                         seL4_RangeError => {
                             Some(RangeError {
                                 min: (*ipcbuf).msg[0],
                                 max: (*ipcbuf).msg[1],
                             })
                         }
                         seL4_AlignmentError => Some(AlignmentError),
                         seL4_FailedLookup => {
                             Some(FailedLookup {
                                 failed_for_source: (*ipcbuf).msg[0] == 1,
                                 lookup_kind: match LookupFailureKind::from_ipcbuf(ipcbuf, 1, 2) {
                                     Some(s) => s,
                                     None => return None,
                                 },
                             })
                         }
                         seL4_TruncatedMessage => Some(TruncatedMessage),
                         seL4_DeleteFirst => Some(DeleteFirst),
                         seL4_RevokeFirst => Some(RevokeFirst),
                         seL4_NotEnoughMemory => {
                             Some(NotEnoughMemory { bytes_available: (*ipcbuf).msg[0] })
                         }
                     }
                }
            }
        }
    }
}

/// Ways capability lookup can fail.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum LookupFailureKind {
    InvalidRoot,
    MissingCapability {
        bits_remaining: seL4_Word,
    },
    DepthMismatch {
        bits_remaining: seL4_Word,
        bits_resolved: seL4_Word,
    },
    GuardMismatch {
        bits_remaining: seL4_Word,
        guard: seL4_Word,
        guard_size: seL4_Word,
    },
}

impl ::core::fmt::Display for LookupFailureKind {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        use LookupFailureKind::*;

        match *self {
            InvalidRoot =>
                write!(f, "root of the address space was not valid (wrong type, not writeable if\
                           the destination, or not readable if the source)"),
            MissingCapability { bits_remaining } =>
                write!(f, "there were {} bits remaining to be resolved, but the slot at that point\
                           was empty",
                       bits_remaining),
            DepthMismatch { bits_remaining, bits_resolved } =>
                write!(f, "after resolving {} bits with {} bits left to resolve, there was no slot\
                           at that depth, or a page cap was found at the wrong depth",
                       bits_resolved, bits_remaining),
            GuardMismatch { bits_remaining, guard, guard_size } =>
                write!(f, "with {} bits left to resolve, there was a guard {:x} (with {} bits)\
                           which did not match",
                       bits_remaining, guard, guard_size),
        }
    }
}

impl LookupFailureKind {
    #[inline(always)]
    unsafe fn from_ipcbuf(ipcbuf: *mut seL4_IPCBuffer, type_idx: usize, details_idx: usize)
                          -> Option<LookupFailureKind> {
        use LookupFailureKind::*;

        let kind = (*ipcbuf).msg[type_idx];
        assert!(kind > seL4_GuardMismatch as seL4_Word, "Unknown lookup failure type");

        // unsafe: transmute could be replaced with an enum_from_primitive!()
        match ::core::mem::transmute::<_, seL4_LookupFailureType>(kind) {
             seL4_NoFailure => None,
             seL4_InvalidRoot => Some(InvalidRoot),
             seL4_MissingCapability => {
                 Some(MissingCapability {
                     bits_remaining: (*ipcbuf).msg[details_idx],
                 })
             }
             seL4_DepthMismatch => {
                 Some(DepthMismatch {
                     bits_remaining: (*ipcbuf).msg[details_idx],
                     bits_resolved: (*ipcbuf).msg[details_idx + 1],
                 })
             }
             seL4_GuardMismatch => {
                 Some(GuardMismatch {
                     bits_remaining: (*ipcbuf).msg[details_idx],
                     guard: (*ipcbuf).msg[details_idx + 1],
                     guard_size: (*ipcbuf).msg[details_idx + 2],
                 })
             }
         }
    }
}
