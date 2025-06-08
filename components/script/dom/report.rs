/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use std::fmt::Display;
use std::time::{SystemTime, UNIX_EPOCH};

use dom_struct::dom_struct;
use servo_url::ServoUrl;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::ReportBinding::ReportMethods;
use crate::dom::bindings::reflector::Reflector;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::cspviolationreportbody::strip_url_for_reports;
use crate::dom::globalscope::GlobalScope;
use crate::dom::reportbody::ReportBody;
use crate::dom::reportingobserver::ReportingObserver;

#[dom_struct]
pub(crate) struct Report {
    reflector_: Reflector,

    /// <https://w3c.github.io/reporting/#report-reporttype>
    #[no_trace]
    type_: ReportType,
    /// <https://w3c.github.io/reporting/#report-url>
    url: DOMString,
    /// <https://w3c.github.io/reporting/#report-body>
    body: DomRefCell<Dom<ReportBody>>,
    /// <https://w3c.github.io/reporting/#report-destination>
    destination: DOMString,
    /// <https://w3c.github.io/reporting/#report-timestamp>
    timestamp: u64,
    /// <https://w3c.github.io/reporting/#report-attempts>
    attempts: u32,
}

#[derive(Clone, MallocSizeOf)]
pub(crate) enum ReportType {
    CSPViolation,
}

impl Display for ReportType {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = match self {
            ReportType::CSPViolation => "csp-violation",
        };
        write!(formatter, "{string}")
    }
}

impl Report {
    fn new_inherited(
        type_: ReportType,
        url: DOMString,
        body: &ReportBody,
        destination: DOMString,
    ) -> Self {
        Self {
            reflector_: Reflector::new(),
            type_,
            url,
            body: DomRefCell::new(Dom::from_ref(body)),
            destination,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            attempts: 0,
        }
    }

    pub(crate) fn copy(&self) -> Report {
        Report::new_inherited(
            self.type_.clone(),
            self.url.clone(),
            &self.body.borrow().copy(),
            self.destination.clone(),
        )
    }

    /// <https://w3c.github.io/reporting/#visible-to-reportingobservers>
    pub(crate) fn is_visible_to_reporting_observers(&self) -> bool {
        match self.type_ {
            // https://w3c.github.io/webappsec-csp/#reporting
            ReportType::CSPViolation => true,
        }
    }

    /// <https://w3c.github.io/reporting/#generate-a-report>
    fn generate_a_report(
        global_scope: &GlobalScope,
        type_: ReportType,
        url: Option<ServoUrl>,
        body: &ReportBody,
        destination: DOMString,
    ) -> Report {
        // Step 2. If url was not provided by the caller, let url be settings’s creation URL.
        let url = url.unwrap_or(global_scope.creation_url().clone());
        // Step 3. Set url’s username to the empty string, and its password to null.
        // Step 4. Set report’s url to the result of executing the URL serializer
        // on url with the exclude fragment flag set.
        let url = strip_url_for_reports(url);
        // Step 1. Let report be a new report object with its values initialized as follows:
        // Step 5. Return report.
        Report::new_inherited(type_, url.into(), body, destination)
    }

    /// <https://w3c.github.io/reporting/#generate-and-queue-a-report>
    pub(crate) fn generate_and_queue_a_report(
        global_scope: &GlobalScope,
        type_: ReportType,
        body: &ReportBody,
        destination: DOMString,
    ) {
        // Step 1. Let settings be context’s relevant settings object.
        // Step 2. Let report be the result of running generate a report with data, type, destination and settings.
        let report = Report::generate_a_report(global_scope, type_, None, body, destination);
        // Step 3. If settings is given, then
        // Step 3.1. Let scope be settings’s global object.
        // Step 3.2. If scope is an object implementing WindowOrWorkerGlobalScope, then
        // execute § 4.2 Notify reporting observers on scope with report with scope and report.
        ReportingObserver::notify_reporting_observers_on_scope(global_scope, &report);
        // Step 4. Append report to context’s reports.
        global_scope.append_report(&report);
    }
}

impl ReportMethods<crate::DomTypeHolder> for Report {
    /// <https://w3c.github.io/reporting/#dom-report-type>
    fn Type(&self) -> DOMString {
        self.type_.to_string().into()
    }

    /// <https://w3c.github.io/reporting/#dom-report-url>
    fn Url(&self) -> DOMString {
        self.url.clone()
    }

    /// <https://w3c.github.io/reporting/#dom-report-body>
    fn GetBody(&self) -> Option<DomRoot<ReportBody>> {
        Some(DomRoot::from_ref(&*self.body.borrow()))
    }
}
