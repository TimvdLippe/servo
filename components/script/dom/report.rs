/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use dom_struct::dom_struct;

use crate::dom::bindings::codegen::Bindings::ReportBinding::ReportMethods;
use crate::dom::bindings::reflector::{Reflector, reflect_dom_object};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::reportbody::ReportBody;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct Report {
    reflector_: Reflector,

    /// <https://w3c.github.io/reporting/#report-reporttype>
    type_: DOMString,
    /// <https://w3c.github.io/reporting/#report-url>
    url: DOMString,
    /// <https://w3c.github.io/reporting/#report-body>
    body: DomRoot<ReportBody>,
}

impl Report {
    fn new_inherited(type_: DOMString, url: DOMString, body: &ReportBody) -> Self {
        Self {
            reflector_: Reflector::new(),
            type_,
            url,
            body: DomRoot::from_ref(body),
        }
    }

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn new(
        type_: DOMString,
        url: DOMString,
        body: &ReportBody,
        global: &GlobalScope,
        can_gc: CanGc,
    ) -> DomRoot<Self> {
        reflect_dom_object(
            Box::new(Self::new_inherited(type_, url, body)),
            global,
            can_gc,
        )
    }
}

impl ReportMethods<crate::DomTypeHolder> for Report {
    /// <https://w3c.github.io/reporting/#dom-report-type>
    fn Type(&self) -> DOMString {
        self.type_.clone()
    }

    /// <https://w3c.github.io/reporting/#dom-report-url>
    fn Url(&self) -> DOMString {
        self.url.clone()
    }

    /// <https://w3c.github.io/reporting/#dom-report-body>
    fn GetBody(&self) -> Option<DomRoot<ReportBody>> {
        Some(self.body.clone())
    }
}
