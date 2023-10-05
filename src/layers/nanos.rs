// SPDX-License-Identifier: BSD-3-Clause
use std::time::{Duration, SystemTime};

use tracing::{Id, Subscriber};
use tracing_subscriber::{layer::Context, registry::LookupSpan, Layer};

#[derive(Debug, Default)]
pub struct NanoCountLayer;

impl<S> Layer<S> for NanoCountLayer
where
    S: Subscriber + for<'lookup> LookupSpan<'lookup>,
{
    fn on_enter(&self, id: &Id, ctx: Context<'_, S>) {
        if let Some(span) = ctx.span(id) {
            span.extensions_mut().insert(SystemTime::now());
        }
    }

    fn on_exit(&self, id: &Id, ctx: Context<'_, S>) {
        if let Some(span) = ctx.span(id) {
            if let Some(time) = span.extensions().get::<SystemTime>() {
                let elapsed = time.elapsed().unwrap_or(Duration::ZERO);
                eprintln!("{}: {}", span.name(), elapsed.as_nanos())
            }
        }
    }
}
