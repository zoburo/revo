use opentelemetry::{KeyValue, global, trace::TracerProvider};
use opentelemetry_sdk::{Resource, metrics::SdkMeterProvider, trace::SdkTracerProvider};
use opentelemetry_stdout::{MetricExporter, SpanExporter};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub struct TelemetryProvider {
    meter_provider: SdkMeterProvider,
    tracer_provider: SdkTracerProvider,
}

impl TelemetryProvider {
    pub fn init() -> Self {
        let resource = Resource::builder()
            .with_service_name(env!("CARGO_PKG_NAME"))
            .with_attributes([
                KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
                KeyValue::new("deployment.environment.name", "development"),
            ])
            .build();

        let meter_provider = SdkMeterProvider::builder()
            .with_resource(resource.clone())
            .with_periodic_exporter(MetricExporter::default())
            .build();
        let tracer_provider = SdkTracerProvider::builder()
            .with_resource(resource)
            .with_simple_exporter(SpanExporter::default())
            .build();

        global::set_meter_provider(meter_provider.clone());
        global::set_tracer_provider(tracer_provider.clone());

        let tracer = tracer_provider.tracer(env!("CARGO_PKG_NAME"));
        let tracing_layer = tracing_opentelemetry::layer().with_tracer(tracer);

        tracing_subscriber::registry()
            .with(tracing_layer)
            .try_init()
            .ok();

        Self {
            meter_provider,
            tracer_provider,
        }
    }

    pub fn shutdown(&self) {
        if let Err(err) = self.meter_provider.shutdown() {
            eprintln!("Error shutting down tracer provider: {err:?}");
        }

        if let Err(err) = self.tracer_provider.shutdown() {
            eprintln!("Error shutting down tracer provider: {err:?}");
        }
    }
}
