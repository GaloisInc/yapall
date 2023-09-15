// SPDX-License-Identifier:i BSD-3-Clause
use std::time::{Duration, SystemTime};

use dashmap::DashMap;
use tracing::{Id, Subscriber};
use tracing_subscriber::{layer::Context, registry::LookupSpan, Layer};

#[derive(Debug, Default)]
pub struct NanoCountLayer {
    durations: DashMap<&'static str, Duration>,
    times: DashMap<Id, SystemTime>,
}

impl NanoCountLayer {
    fn report(&self) {
        for tup in &self.durations {
            eprintln!("{}: {}", tup.key(), tup.value().as_nanos())
        }
        self.durations.clear();
    }
}

impl<S> Layer<S> for NanoCountLayer
where
    S: Subscriber + for<'lookup> LookupSpan<'lookup>,
{
    fn on_enter(&self, id: &Id, _ctx: Context<'_, S>) {
        self.times.insert(id.clone(), SystemTime::now());
    }

    fn on_exit(&self, id: &Id, ctx: Context<'_, S>) {
        if let Some(span) = ctx.span(id) {
            *self.durations.entry(span.name()).or_insert(Duration::ZERO) +=
                self.times.get(id).unwrap().elapsed().unwrap();
            self.times.remove(id);
            // TODO: Wasteful to do this each time, should do it all at once
            // at the end. However, implementing `Drop` didn't seem to do the
            // trick.
            self.report();
        }
    }
}
