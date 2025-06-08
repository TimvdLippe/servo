/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;
use std::rc::Rc;

use dom_struct::dom_struct;
use js::rust::HandleObject;
use script_bindings::str::DOMString;

use crate::dom::bindings::callback::ExceptionHandling;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::ReportBinding::ReportMethods;
use crate::dom::bindings::codegen::Bindings::ReportingObserverBinding::{
    ReportList, ReportingObserverCallback, ReportingObserverMethods, ReportingObserverOptions,
};
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::{DomGlobal, Reflector, reflect_dom_object_with_proto};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::globalscope::GlobalScope;
use crate::dom::report::Report;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct ReportingObserver {
    reflector_: Reflector,

    #[ignore_malloc_size_of = "Rc has unclear ownership"]
    callback: Rc<ReportingObserverCallback>,
    buffered: RefCell<bool>,
    types: DomRefCell<Vec<DOMString>>,
    report_queue: DomRefCell<Vec<Dom<Report>>>,
}

impl ReportingObserver {
    fn new_inherited(
        callback: Rc<ReportingObserverCallback>,
        options: &ReportingObserverOptions,
    ) -> Self {
        Self {
            reflector_: Reflector::new(),
            callback,
            buffered: RefCell::new(options.buffered),
            types: DomRefCell::new(options.types.clone().unwrap_or_default()),
            report_queue: Default::default(),
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

    /// <https://w3c.github.io/reporting/#add-report>
    fn add_report_to_observer(&self, report: &Report) {
        // Step 1. If report’s type is not visible to ReportingObservers, return.
        if !report.is_visible_to_reporting_observers() {
            return;
        }
        // Step 2. If observer’s options has a non-empty types member which does not contain report’s type, return.
        let types = self.types.borrow();
        if !types.is_empty() && !types.contains(&report.Type()) {
            return;
        }
        // Step 3. Create a new Report r with type initialized to report’s type,
        // url initialized to report’s url, and body initialized to report’s body.
        let report = report.copy();
        // Step 4. Append r to observer’s report queue.
        self.report_queue.borrow_mut().push(Dom::from_ref(&report));
        // Step 5. If the size of observer’s report queue is 1:
        if self.report_queue.borrow().len() == 1 {
            // Step 5.1. Let global be observer’s relevant global object.
            let global = self.global();
            // Step 5.2. Queue a task to § 4.4 Invoke reporting observers with notify list
            // with a copy of global’s registered reporting observer list.
            let observers_global = Trusted::new(&*global);
            global.task_manager().dom_manipulation_task_source().queue(
                task!(notify_reporting_observers: move || {
                    ReportingObserver::invoke_reporting_observers_with_notify_list(
                        observers_global.root().registered_reporting_observers()
                    );
                }),
            );
        }
    }

    /// <https://w3c.github.io/reporting/#notify-observers>
    pub(crate) fn notify_reporting_observers_on_scope(global_scope: &GlobalScope, report: &Report) {
        // Step 1. For each ReportingObserver observer registered with scope,
        // execute § 4.3 Add report to observer on report and observer.
        for observer in global_scope.registered_reporting_observers().iter() {
            observer.add_report_to_observer(report);
        }
        // Step 2. Append report to scope’s report buffer.
        // Step 3. Let type be report’s type.
        // Step 4. If scope’s report buffer now contains more than 100 reports with
        // type equal to type, remove the earliest item with type equal to type in the report buffer.
    }

    /// <https://w3c.github.io/reporting/#invoke-observers>
    fn invoke_reporting_observers_with_notify_list(notify_list: Vec<DomRoot<ReportingObserver>>) {
        // Step 1. For each ReportingObserver observer in notify list:
        for observer in notify_list.iter() {
            // Step 1.1. If observer’s report queue is empty, then continue.
            if observer.report_queue.borrow().is_empty() {
                continue;
            }
            // Step 1.2. Let reports be a copy of observer’s report queue
            // Step 1.3. Empty observer’s report queue
            let reports = std::mem::take(&mut *observer.report_queue.borrow_mut());
            // Step 1.4. Invoke observer’s callback with « reports, observer » and "report",
            // and with observer as the callback this value.
            let _ = observer.callback.Call_(
                &**observer,
                // TODO: Figure out why this line below panics
                // reports.iter().map(|r| DomRoot::from_ref(&*r)).collect(),
                vec![],
                observer,
                ExceptionHandling::Report,
                CanGc::note(),
            );
        }
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
        // Step 1. Let reports be a copy of this’s report queue.
        // Step 2. Empty this’s report queue.
        let reports = std::mem::take(&mut *self.report_queue.borrow_mut());
        // Step 3. Return reports.
        reports
            .into_iter()
            .map(|r| DomRoot::from_ref(&*r))
            .collect()
    }
}
