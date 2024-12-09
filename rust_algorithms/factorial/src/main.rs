#![deny(warnings)]

// This code is copied and adapted from the hyper.rs guides.
// Last retrieved on Sun 19th May 2024 at: https://hyper.rs/guides/1/server/hello-world/

use std::convert::Infallible;
use std::net::SocketAddr;

use http_body_util::Full;
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;

fn factorial(n: u64) -> u64 {
    // Initialize the result variable with 1
    let mut result = 1;

    // Start a for loop to calculate the factorial
    for i in 1..=n {
        // Multiply the result by the current value of 'i'
        result *= i;
    }

    // Return the calculated factorial
    result
}

// An async function that consumes a request, does nothing with it and returns a
// response.
async fn rustless(req: Request<impl hyper::body::Body>) -> Result<Response<Full<Bytes>>, Infallible> {

    // Extract the query string from the request
    let query = req.uri().query().unwrap_or("");

    // Parse the query to get the "number" parameter
    let mut number = 12; // Default number for factorial
    for (key, value) in url::form_urlencoded::parse(query.as_bytes()) {
        if key == "number" {
            number = value.parse().unwrap_or(12);
        }
    }

    // Calculate the factorial
    let result = factorial(number);

    // Format the result as a string
    let response_text = format!("Factorial of {} is: {}", number, result);

    Ok(Response::new(Full::new(Bytes::from(response_text))))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));

    // We create a TcpListener and bind it to 0.0.0.0:8080
    let listener = TcpListener::bind(addr).await?;

    // We start a loop to continuously accept incoming connections
    loop {
        let (stream, _) = listener.accept().await?;

        // Use an adapter to access something implementing `tokio::io` traits as if they implement
        // `hyper::rt` IO traits.
        let io = TokioIo::new(stream);

        // Spawn a tokio task to serve multiple connections concurrently
        tokio::task::spawn(async move {
            // Finally, we bind the incoming connection to our `hello` service
            if let Err(err) = http1::Builder::new()
                // `service_fn` converts our function in a `Service`
                .serve_connection(io, service_fn(rustless))
                .await
            {
                eprintln!("Error serving connection: {:?}", err);
            }
        });
    }
}