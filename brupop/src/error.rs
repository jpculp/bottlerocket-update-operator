use actix_web::error;
use snafu::Snafu;

/// The crate-wide result type.
pub type Result<T> = std::result::Result<T, Error>;

/// The crate-wide error type.
#[derive(Debug, Snafu)]
#[snafu(visibility = "pub")]
pub enum Error {
    #[snafu(display("Unable to create client: {}", source))]
    ClientCreate { source: kube::Error },

    #[snafu(display("Error creating {}: {}", what, source))]
    Creation { what: String, source: kube::Error },

    #[snafu(display("Error running HTTP server: {}", source))]
    HttpServerError { source: std::io::Error },
}

impl error::ResponseError for Error {}
