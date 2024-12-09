#![deny(warnings)]

// This code is copied and adapted from the hyper.rs guides.
// Last retrieved on Sun 19th May 2024 at: https://hyper.rs/guides/1/server/hello-world/

// The Computer Language Benchmarks Game
// https://salsa.debian.org/benchmarksgame-team/benchmarksgame/
//
// contributed by Matt Watson
// contributed by TeXitoi
// contributed by Volodymyr M. Lisivka
// contributed by Michael Ciccotti

extern crate generic_array;
extern crate num_traits;
extern crate numeric_array;
extern crate rayon;

use generic_array::typenum::consts::U8;
use numeric_array::NumericArray as Arr;
use rayon::prelude::*;

use std::convert::Infallible;
use std::net::SocketAddr;
use std::io::stdout;
use std::io::Write;

use http_body_util::Full;
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;



// [f64;8]
type Vecf64 = Arr<f64, U8>;
type Constf64 = numeric_array::NumericConstant<f64>;

const MAX_ITER: usize = 50;
const VLEN: usize = 8;

#[inline(always)]
pub fn mbrot8(out: &mut u8, cr: Vecf64, ci: Constf64) {
    let mut zr = Arr::splat(0f64);
    let mut zi = Arr::splat(0f64);
    let mut tr = Arr::splat(0f64);
    let mut ti = Arr::splat(0f64);
    let mut absz = Arr::splat(0f64);

    for _ in 0..MAX_ITER / 5 {
        for _ in 0..5 {
            zi = (zr + zr) * zi + ci;
            zr = tr - ti + cr;
            tr = zr * zr;
            ti = zi * zi;
        }

        absz = tr + ti;
        if absz.iter().all(|&t| t > 4.) {
            return;
        }
    }

    *out = absz.iter().enumerate().fold(0, |accu, (i, &t)| {
        accu | if t <= 4. { 0x80 >> i } else { 0 }
    });
}

// An async function that consumes a request, does nothing with it and returns a
// response.
async fn rustless(_: Request<impl hyper::body::Body>) -> Result<Response<Full<Bytes>>, Infallible> {

    let size = 16000;
    // Round size to multiple of 8
    let size = size / VLEN * VLEN;

    let inv = 2. / size as f64;

    let mut xloc = vec![Arr::splat(0f64); size / VLEN];
    for i in 0..size {
        xloc[i / VLEN][i % VLEN] = i as f64 * inv - 1.5;
    }


    let mut rows = vec![0; size * size / VLEN];
    rows.par_chunks_mut(size / VLEN)
        .enumerate()
        .for_each(|(y, out)| {
            let ci = numeric_array::NumericConstant(y as f64 * inv - 1.);
            out.iter_mut()
                .enumerate()
                .for_each(|(i, inner_out)| mbrot8(inner_out, xloc[i], ci));
        });

    rows.par_chunks_mut(size / VLEN)
        .enumerate()
        .for_each(|(y, out)| {
            let ci = numeric_array::NumericConstant(y as f64 * inv - 1.);
            out.iter_mut()
                .enumerate()
                .for_each(|(i, inner_out)| mbrot8(inner_out, xloc[i], ci));
        });
//     let _ = stdout().write_all(&rows);

        let body = format!("P4\n{} {}\nComplete", size, size);


    Ok(Response::new(Full::new(Bytes::from(body))))
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