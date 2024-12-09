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


// Function to find the last prime number up to `max_number`
fn find_last_prime(max_number: u32) -> u32 {
    let mut last_prime = 0;

    for i in 1..=max_number {
        if is_prime(i) {
            last_prime = i;
        }
    }

    last_prime
}

// Function to check if a number is prime
fn is_prime(n: u32) -> bool {
    if n < 2 {
        return false; // Numbers less than 2 are not prime
    }

    for i in 2..=((n as f64).sqrt() as u32) {
        if n % i == 0 {
            return false;
        }
    }
    true
}

// An async function that consumes a request, does nothing with it and returns a
// response.
async fn rustless(req: Request<impl hyper::body::Body>) -> Result<Response<Full<Bytes>>, Infallible> {

    // Extract the query string from the request
    let query = req.uri().query().unwrap_or("");

    // Parse the query to get the "number" parameter
    let mut number = 100; // Default number for factorial
    for (key, value) in url::form_urlencoded::parse(query.as_bytes()) {
        if key == "number" {
            number = value.parse().unwrap_or(100);
        }
    }

    let result = find_last_prime(number);

    // Format the result as a string
    let response_text = format!("The last prime number up to {} is: {}", number, result);

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