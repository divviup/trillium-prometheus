//! This is a small utility crate that provides a Prometheus metrics endpoint as a Trillium handler.
//! It responds to GET requests to "/metrics" with metrics from the provided registry, using
//! text-format encoding.
//!
//! # Example:
//!
//! ```
//! # let stopper = trillium_smol::Stopper::new();
//! # stopper.stop();
//! let registry = prometheus::Registry::new();
//! let handler = trillium_prometheus::text_format_handler(registry);
//! trillium_smol::config()
//!     .with_host("0.0.0.0")
//!     .with_port(9464)
//! #   .with_stopper(stopper)
//!     .run(handler);
//! ```
use prometheus::{Encoder, Registry, TextEncoder};
use tracing::error;
use trillium::{KnownHeaderName, Status};
use trillium_router::Router;

/// Creates a handler that responds to GET requests for "/metrics".
pub fn text_format_handler(registry: Registry) -> Router {
    Router::new().get("metrics", move |conn: trillium::Conn| {
        let registry = registry.clone();
        async move {
            let mut buffer = Vec::new();
            let encoder = TextEncoder::new();
            match encoder.encode(&registry.gather(), &mut buffer) {
                Ok(()) => conn
                    .with_response_header(
                        KnownHeaderName::ContentType,
                        encoder.format_type().to_owned(),
                    )
                    .ok(buffer),
                Err(error) => {
                    error!(%error, "Failed to encode Prometheus metrics");
                    conn.with_status(Status::InternalServerError)
                }
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use prometheus::{IntGauge, Registry};
    use trillium_testing::{assert_response, prelude::get};

    use crate::text_format_handler;

    #[test]
    fn text_format_encode() {
        let registry = Registry::new();
        let gauge = IntGauge::new("my_gauge", "Test fixture").unwrap();
        gauge.set(5);
        registry.register(Box::new(gauge)).unwrap();

        let handler = text_format_handler(registry);
        assert_response!(
            get("metrics").on(&handler),
            200,
            "# HELP my_gauge Test fixture\n# TYPE my_gauge gauge\nmy_gauge 5"
        );
    }
}
