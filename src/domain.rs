// Copyright (c) 2015 The Robigalia Project Developers
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>,
// at your option. All files in the project carrying such
// notice may not be copied, modified, or distributed except
// according to those terms.

//! Controlling the domain of threads.

use sel4_sys::*;

cap_wrapper!{
    :DomainSet
}

impl DomainSet {
    /// Change the domain of a thread.
    pub fn set(&self, domain: u8, thread: ::Thread) -> ::Result {
        thread.set_domain(domain, *self)
    }
}
