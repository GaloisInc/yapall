// SPDX-License-Identifier:i BSD-3-Clause
use tracing::{Id, Subscriber};
use tracing_subscriber::{layer::Context, registry::LookupSpan, Layer};

#[derive(Debug, Default)]
pub struct ExecCountLayer {}

impl<S> Layer<S> for ExecCountLayer
where
    S: Subscriber + for<'lookup> LookupSpan<'lookup>,
{
    fn on_exit(&self, id: &Id, ctx: Context<'_, S>) {
        if let Some(span) = ctx.span(id) {
            eprintln!("{} 1", span.name());
        }
    }
}
