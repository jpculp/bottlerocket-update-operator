use std::{convert::TryFrom, env};

use controller::{telemetry::vending_metrics, BrupopController};
use models::{
    constants::{CONTROLLER_INTERNAL_PORT, NAMESPACE},
    node::{BottlerocketShadow, K8SBottlerocketShadowClient},
};

use actix_web::{web::Data, App, HttpServer};

use futures::StreamExt;
use k8s_openapi::api::core::v1::Node;
use kube::{
    api::{Api, ListParams},
    runtime::{reflector, watcher::watcher, WatchStreamExt},
    ResourceExt,
};

use opentelemetry::sdk::export::metrics::aggregation;
use opentelemetry::sdk::metrics::{controllers, processors, selectors};
use opentelemetry::sdk::propagation::TraceContextPropagator;
use snafu::ResultExt;
use tracing::{event, Level};
use tracing_subscriber::{fmt, layer::SubscriberExt, EnvFilter, Registry};

const DEFAULT_TRACE_LEVEL: &str = "info";

/// The module-wide result type.
type Result<T> = std::result::Result<T, controller_error::Error>;

#[actix_web::main]
async fn main() -> Result<()> {
    init_telemetry()?;

    let incluster_config =
        kube::Config::incluster_dns().context(controller_error::ConfigCreateSnafu)?;

    let k8s_client = kube::client::Client::try_from(incluster_config)
        .context(controller_error::ClientCreateSnafu)?;

    // The `BrupopController` needs a `reflector::Store`, which is updated by a reflector
    // that runs concurrently. We'll create the store and run the reflector here.
    let brss = Api::<BottlerocketShadow>::namespaced(k8s_client.clone(), NAMESPACE);
    let brs_store = reflector::store::Writer::<BottlerocketShadow>::default();
    let brs_reader = brs_store.as_reader();

    let node_client = K8SBottlerocketShadowClient::new(k8s_client.clone());

    let controller = controllers::basic(
        processors::factory(
            selectors::simple::histogram([1.0, 2.0, 5.0, 10.0, 20.0, 50.0]),
            aggregation::cumulative_temporality_selector(),
        )
        .with_memory(true),
    )
    .build();

    // Exporter has to be initialized before BrupopController
    // in order to setup global meter provider properly
    let exporter = opentelemetry_prometheus::exporter(controller).init();

    // Setup and run a reflector, ensuring that `BottlerocketShadow` updates are reflected to the controller.
    let brs_reflector = reflector::reflector(brs_store, watcher(brss, ListParams::default()));
    let brs_drainer = brs_reflector
        .touched_objects()
        .filter_map(|x| async move { std::result::Result::ok(x) })
        .for_each(|brs| {
            event!(
                Level::TRACE,
                brs_name = %brs.name_any(),
                "Processed a k8s event for a BottlerocketShadow object."
            );
            futures::future::ready(())
        });

    let nodes: Api<Node> = Api::all(k8s_client.clone());
    let nodes_store = reflector::store::Writer::<Node>::default();
    let node_reader = nodes_store.as_reader();
    let node_reflector = reflector::reflector(nodes_store, watcher(nodes, ListParams::default()));
    let node_drainer = node_reflector
        .touched_objects()
        .filter_map(|x| async move { std::result::Result::ok(x) })
        .for_each(|_node| {
            event!(Level::DEBUG, "Processed event for node");
            futures::future::ready(())
        });

    // Setup and run the controller.
    let controller = BrupopController::new(k8s_client, node_client, brs_reader, node_reader);
    let controller_runner = controller.run();

    let k8s_service_addr = env::var("KUBERNETES_SERVICE_HOST")
        .context(controller_error::MissingClusterIPFamilySnafu)?;
    let bindaddress = if k8s_service_addr.contains(':') {
        // IPv6 format
        "[::]"
    } else {
        // IPv4 format
        "0.0.0.0"
    };

    // Setup Http server to vend prometheus metrics
    let prometheus_server = HttpServer::new(move || {
        App::new()
            .app_data(Data::new(exporter.clone()))
            .service(vending_metrics)
    })
    .bind(format!("{}:{}", bindaddress, CONTROLLER_INTERNAL_PORT))
    .context(controller_error::PrometheusServerSnafu)?
    .run();

    tokio::select! {
        _ = brs_drainer => {
            event!(Level::ERROR, "brs reflector drained");
        },
        _ = node_drainer => {
            event!(Level::ERROR, "node reflector drained");
        },
        controller = controller_runner => {
            event!(Level::ERROR, "controller exited");
            controller.context(controller_error::ControllerSnafu)?
        },
        _ = prometheus_server => {
            event!(Level::ERROR, "metric server exited");
        }
    };
    Ok(())
}

fn init_telemetry() -> Result<()> {
    opentelemetry::global::set_text_map_propagator(TraceContextPropagator::new());

    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(DEFAULT_TRACE_LEVEL));
    let stdio_formatting_layer = fmt::layer().pretty();
    let subscriber = Registry::default()
        .with(env_filter)
        .with(stdio_formatting_layer);
    tracing::subscriber::set_global_default(subscriber)
        .context(controller_error::TracingConfigurationSnafu)?;

    Ok(())
}

pub mod controller_error {
    use controller::controllerclient_error;
    use snafu::Snafu;

    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub))]
    pub enum Error {
        #[snafu(display("Unable to create client config: '{}'", source))]
        ConfigCreate {
            source: kube::config::InClusterError,
        },

        #[snafu(display("Unable to create client: '{}'", source))]
        ClientCreate { source: kube::Error },

        #[snafu(display("Error running controller server: '{}'", source))]
        ControllerError {
            source: controllerclient_error::Error,
        },

        #[snafu(display("Unable to get associated node name: {}", source))]
        GetNodeName { source: std::env::VarError },

        #[snafu(display("The Kubernetes WATCH on {} objects has failed.", object))]
        KubernetesWatcherFailed { object: String },

        #[snafu(display("Error configuring tracing: '{}'", source))]
        TracingConfiguration {
            source: tracing::subscriber::SetGlobalDefaultError,
        },

        #[snafu(display("Error determining the cluster server address: '{}'", source))]
        MissingClusterIPFamily { source: std::env::VarError },

        #[snafu(display("Error running prometheus HTTP server: '{}'", source))]
        PrometheusServerError { source: std::io::Error },

        #[snafu(display("Failed to run prometheus on controller"))]
        PrometheusError,
    }
}
