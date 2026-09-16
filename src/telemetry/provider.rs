use anyhow::Result;
use once_cell::sync::OnceCell;
use opentelemetry::{KeyValue, global, trace::TracerProvider};
use opentelemetry_sdk::{Resource, metrics::SdkMeterProvider, trace::SdkTracerProvider};
use opentelemetry_semantic_conventions::{
    resource::DEPLOYMENT_ENVIRONMENT_NAME, trace::SERVICE_VERSION,
};
use opentelemetry_stdout::{MetricExporter, SpanExporter};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::core::{config, constant};

pub struct TelemetryProvider {
    meter_provider: SdkMeterProvider,
    tracer_provider: SdkTracerProvider,
}

impl TelemetryProvider {
    pub fn init() -> Result<Self> {
        let service_name = constant::SERVICE_NAME;
        let service_version = constant::SERVICE_VERSION;
        let environment: &str = &config().environment;

        let resource = Resource::builder()
            .with_service_name(service_name)
            .with_attributes([
                KeyValue::new(SERVICE_VERSION, service_version),
                KeyValue::new(DEPLOYMENT_ENVIRONMENT_NAME, environment),
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

        let tracer = tracer_provider.tracer(service_name);
        let tracing_layer = tracing_opentelemetry::layer().with_tracer(tracer);

        tracing_subscriber::registry()
            .with(tracing_layer)
            .try_init()?;

        Ok(Self {
            meter_provider,
            tracer_provider,
        })
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

static TELEMETRY_PROVIDER: OnceCell<TelemetryProvider> = OnceCell::new();

pub fn telemetry_provider() -> Result<&'static TelemetryProvider> {
    TELEMETRY_PROVIDER.get_or_try_init(TelemetryProvider::init)
}
