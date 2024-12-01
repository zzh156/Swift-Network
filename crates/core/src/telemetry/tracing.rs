use opentelemetry::{
    trace::{Span as OtelSpan, Tracer as OtelTracer},
    Context, KeyValue,
};
use std::time::Instant;

pub struct Tracer {
    enabled: bool,
    tracer: OtelTracer,
}

pub struct Span {
    inner: Option<OtelSpan>,
    start_time: Instant,
    name: String,
}

#[derive(Debug, Clone)]
pub struct SpanContext {
    trace_id: String,
    span_id: String,
    parent_id: Option<String>,
}

impl Tracer {
    pub fn new(enabled: bool) -> Self {
        let tracer = if enabled {
            // 初始化 OpenTelemetry tracer
            opentelemetry::global::tracer("sui")
        } else {
            opentelemetry::global::noop_tracer()
        };

        Self {
            enabled,
            tracer,
        }
    }

    pub fn start_span(&self, name: &str) -> Span {
        if !self.enabled {
            return Span::new_disabled(name);
        }

        let span = self.tracer
            .span_builder(name)
            .with_start_time(opentelemetry::time::now())
            .start(&self.tracer);

        Span {
            inner: Some(span),
            start_time: Instant::now(),
            name: name.to_string(),
        }
    }

    pub fn current_context(&self) -> Option<SpanContext> {
        if !self.enabled {
            return None;
        }

        let ctx = Context::current();
        let span = ctx.span();
        
        Some(SpanContext {
            trace_id: span.span_context().trace_id().to_string(),
            span_id: span.span_context().span_id().to_string(),
            parent_id: None, // 可以从上下文中获取
        })
    }
}

impl Span {
    fn new_disabled(name: &str) -> Self {
        Self {
            inner: None,
            start_time: Instant::now(),
            name: name.to_string(),
        }
    }

    pub fn add_event(&self, name: &str, attributes: Vec<KeyValue>) {
        if let Some(span) = &self.inner {
            span.add_event(name.to_string(), attributes);
        }
    }

    pub fn set_attribute(&self, key: &str, value: &str) {
        if let Some(span) = &self.inner {
            span.set_attribute(KeyValue::new(key, value.to_string()));
        }
    }

    pub fn record_error(&self, error: &dyn std::error::Error) {
        if let Some(span) = &self.inner {
            span.record_error(error);
        }
    }
}

impl Drop for Span {
    fn drop(&mut self) {
        if let Some(span) = self.inner.take() {
            let duration = self.start_time.elapsed();
            span.set_attribute(KeyValue::new("duration_ms", duration.as_millis() as i64));
            span.end();
        }
    }
}