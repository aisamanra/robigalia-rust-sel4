// Copyright (c) 2015 The Robigalia Project Developers
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>,
// at your option. All files in the project carrying such
// notice may not be copied, modified, or distributed except
// according to those terms.

#[macro_export]
macro_rules! println {
    ($($toks:tt)*) => ({
        use ::core::fmt::Write;
        let _ = writeln!($crate::DebugOutHandle, $($toks)*);
    })
}

macro_rules! unsafe_as_result {
    // already in an unsafe block
    (@ $e:expr) => {
        if { $e } == 0 {
            Ok(())
        } else {
            Err(::Error(::GoOn::CheckIPCBuf))
        }
    };
    ($e:expr) => {
        if unsafe { $e } == 0 {
            Ok(())
        } else {
            Err(::Error(::GoOn::CheckIPCBuf))
        }
    }
}

macro_rules! cap_wrapper {
    (()) => {};
    (($($attrs:tt)*) #[$meta:meta] $($tail:tt)*) => {
        cap_wrapper!{ ($($attrs)* #[$meta]) $($tail)* }
    };
    (($($meta:tt)*) $name:ident $(= $objtag:ident $size:expr)*, $($tail:tt)*) => {
        #[derive(Debug, Copy, Clone, PartialEq, Eq)]
        $($meta)*
        pub struct $name {
            cptr: ::sel4_sys::seL4_CPtr,
        }

        impl ::ToCap for $name {
            #[inline(always)]
            fn to_cap(&self) -> ::sel4_sys::seL4_CPtr {
                self.cptr.to_cap()
            }
        }

        impl $name {
            #[inline(always)]
            pub const fn from_cap(cptr: ::sel4_sys::seL4_CPtr) -> Self {
                $name { cptr: cptr }
            }
        }

        $(
            impl ::Allocatable for $name {
                fn create(untyped_memory: ::sel4_sys::seL4_CPtr, mut dest: ::cspace::Window,
                          size_bits: ::sel4_sys::seL4_Word) -> ::Result
                {
                    use ::ToCap;
                    use ::CONFIG_RETYPE_FAN_OUT_LIMIT;

                    // Most we can create in one syscall is CONFIG_RETYPE_FAN_OUT_LIMIT (256)
                    while dest.num_slots > CONFIG_RETYPE_FAN_OUT_LIMIT {
                        unsafe_as_result!(::sel4_sys::seL4_Untyped_Retype(
                            untyped_memory,
                            $objtag as seL4_Word,
                            size_bits,
                            dest.cnode.root.to_cap(),
                            dest.cnode.cptr,
                            dest.cnode.depth as seL4_Word,
                            dest.first_slot_idx,
                            CONFIG_RETYPE_FAN_OUT_LIMIT,
                        ))?;
                        dest.first_slot_idx += CONFIG_RETYPE_FAN_OUT_LIMIT;
                        dest.num_slots -= CONFIG_RETYPE_FAN_OUT_LIMIT;
                    }

                    if dest.num_slots > 0 {
                        unsafe_as_result!(::sel4_sys::seL4_Untyped_Retype(
                            untyped_memory,
                            $objtag as seL4_Word,
                            size_bits,
                            dest.cnode.root.to_cap(),
                            dest.cnode.cptr,
                            dest.cnode.depth as seL4_Word,
                            dest.first_slot_idx,
                            dest.num_slots,
                        ))?;
                    }

                    Ok(())
                }

                fn object_size(size_bits: seL4_Word) -> isize {
                    $size(size_bits) as isize
                }
            }
        )*

        cap_wrapper!{ () $($tail)* }
    };
}
