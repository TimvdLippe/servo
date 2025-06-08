/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use dom_struct::dom_struct;

use crate::dom::bindings::codegen::Bindings::ReportingObserverBinding::ReportBodyMethods;
use crate::dom::bindings::reflector::{Reflector, reflect_dom_object};
use crate::dom::bindings::root::DomRoot;
use crate::dom::globalscope::GlobalScope;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct ReportBody {
    reflector_: Reflector,

    body: String,
}

impl ReportBody {
    fn new_inherited(body: String) -> Self {
        Self {
            reflector_: Reflector::new(),
            body,
        }
    }

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn new(body: String, global: &GlobalScope, can_gc: CanGc) -> DomRoot<Self> {
        reflect_dom_object(Box::new(Self::new_inherited(body)), global, can_gc)
    }
}

impl ReportBodyMethods<crate::DomTypeHolder> for ReportBody {}
