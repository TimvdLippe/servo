/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;

use dom_struct::dom_struct;
use js::rust::HandleObject;
use script_bindings::str::DOMString;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::ReportingObserverBinding::{
    ReportList, ReportingObserverCallback, ReportingObserverMethods, ReportingObserverOptions,
};
use crate::dom::bindings::reflector::{DomGlobal, Reflector, reflect_dom_object_with_proto};
use crate::dom::bindings::root::DomRoot;
use crate::dom::globalscope::GlobalScope;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct ReportingObserver {
    reflector_: Reflector,

    #[ignore_malloc_size_of = "Rc has unclear ownership"]
    callback: Rc<ReportingObserverCallback>,
    buffered: DomRefCell<bool>,
    types: DomRefCell<Vec<DOMString>>,
}

impl ReportingObserver {
    fn new_inherited(
        callback: Rc<ReportingObserverCallback>,
        options: &ReportingObserverOptions,
    ) -> Self {
        Self {
            reflector_: Reflector::new(),
            callback,
            buffered: DomRefCell::new(options.buffered),
            types: DomRefCell::new(options.types.clone().unwrap_or_default()),
        }
    }

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn new_with_proto(
        callback: Rc<ReportingObserverCallback>,
        options: &ReportingObserverOptions,
        global: &GlobalScope,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> DomRoot<Self> {
        reflect_dom_object_with_proto(
            Box::new(Self::new_inherited(callback, options)),
            global,
            proto,
            can_gc,
        )
    }
}

impl ReportingObserverMethods<crate::DomTypeHolder> for ReportingObserver {
    /// <https://w3c.github.io/reporting/#dom-reportingobserver-reportingobserver>
    fn Constructor(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        can_gc: CanGc,
        callback: Rc<ReportingObserverCallback>,
        options: &ReportingObserverOptions,
    ) -> DomRoot<ReportingObserver> {
        // Step 1. Create a new ReportingObserver object observer.
        // Step 2. Set observer’s callback to callback.
        // Step 3. Set observer’s options to options.
        // Step 4. Return observer.
        ReportingObserver::new_with_proto(callback, options, global, proto, can_gc)
    }

    /// <https://w3c.github.io/reporting/#dom-reportingobserver-observe>
    fn Observe(&self) {
        // Step 1. Let global be the be the relevant global object of this.
        let global = &self.global();
        // Step 2. Append this to the global’s registered reporting observer list.
        global.append_reporting_observer(self);
        // Step 3. If this’s buffered option is false, return.
        if !*self.buffered.borrow() {
            return;
        }
        // Step 4. Set this’s buffered option to false.
        *self.buffered.borrow_mut() = false;
        // Step 5.For each report in global’s report buffer, queue a task to
        // execute § 4.3 Add report to observer with report and this.
        // TODO(37328)
    }

    /// <https://w3c.github.io/reporting/#dom-reportingobserver-disconnect>
    fn Disconnect(&self) {
        // Step 1. If this is not registered, return.
        // Skipped, as this is handled in `remove_reporting_observer`

        // Step 2. Let global be the relevant global object of this.
        let global = &self.global();
        // Step 3. Remove this from global’s registered reporting observer list.
        global.remove_reporting_observer(self);
    }

    /// <https://w3c.github.io/reporting/#dom-reportingobserver-takerecords>
    fn TakeRecords(&self) -> ReportList {
        unimplemented!()
    }
}
