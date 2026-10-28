use std::{fmt, sync::OnceLock};
use tracing::{Event, Level, Subscriber, subscriber::SetGlobalDefaultError};
use tracing_subscriber::{
    fmt::{
        FmtContext, FormatEvent, FormatFields,
        format::{self, FmtSpan, Writer},
        writer::MakeWriterExt,
    },
    registry::LookupSpan,
};

static INITIALIZED: OnceLock<Result<(), SetGlobalDefaultError>> = OnceLock::new();

struct Indented;

impl<S, N> FormatEvent<S, N> for Indented
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        context: &FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &Event<'_>,
    ) -> fmt::Result {
        // Only add nesting; the standard formatter owns timestamps, fields,
        // and lifecycle events, including timing for spans without a return.
        if let Some(scope) = context.event_scope() {
            for _ in scope.skip(1) {
                writer.write_str("  ")?;
            }
        }
        format::format()
            .with_target(false)
            .format_event(context, writer, event)
    }
}

// The native loader calls this before publishing the collector to CoreCLR.
// Cache the result so repeated probes cannot install a second subscriber.
pub(crate) fn init() -> Result<(), &'static SetGlobalDefaultError> {
    INITIALIZED
        .get_or_init(|| {
            let level = std::env::var("RUST_GC_LOG_LEVEL")
                .ok()
                .and_then(|value| value.parse::<Level>().ok())
                .unwrap_or(Level::INFO);
            let writer = std::io::stderr
                .with_max_level(Level::ERROR)
                .or_else(std::io::stdout);
            let subscriber = tracing_subscriber::fmt()
                .with_max_level(level)
                .with_writer(writer)
                .with_ansi(false)
                .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
                .event_format(Indented)
                .finish();

            // Return initialization failures to the loader instead of panicking
            // at an extern-C boundary or silently dropping diagnostic output.
            tracing::subscriber::set_global_default(subscriber)
        })
        .as_ref()
        .copied()
}
