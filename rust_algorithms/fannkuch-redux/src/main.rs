#![deny(warnings)]

// This code is copied and adapted from the hyper.rs guides.
// Last retrieved on Sun 19th May 2024 at: https://hyper.rs/guides/1/server/hello-world/

// The Computer Language Benchmarks Game
// https://salsa.debian.org/benchmarksgame-team/benchmarksgame/
//
// Contributed by Cliff L. Biffle, translated from Jeremy Zerfas's C program.
//
// The C program was based on the Ada program by Jonathan Parker and Georg
// Bauhaus which in turn was based on code by Dave Fladebo, Eckehard Berns,
// Heiner Marxen, Hongwei Xi, and The Anh Tran and also the Java program by Oleg
// Mazurov.

extern crate rayon;

use rayon::prelude::*;
use std::convert::Infallible;
use std::mem::replace;
use std::net::SocketAddr;

use http_body_util::Full;
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;

// This value controls how many blocks the workload is broken up into (as long
// as the value is less than or equal to the factorial of the argument to this
// program) in order to allow the blocks to be processed in parallel if
// possible. PREFERRED_NUMBER_OF_BLOCKS_TO_USE should be some number which
// divides evenly into all factorials larger than it. It should also be around
// 2-8 times the amount of threads you want to use in order to create enough
// blocks to more evenly distribute the workload amongst the threads.
const PREFERRED_NUMBER_OF_BLOCKS_TO_USE: usize = 12;

// One greater than the maximum `n` value. Used to size stack arrays.
const MAX_N: usize = 16;

fn compute_fannkuch_redux(n: usize) -> (usize, usize) {
    // This assert eliminates several bounds checks.
    assert!(n < MAX_N);

    // Create and initialize factorial_lookup_table.
    let factorial_lookup_table = {
        let mut table: [usize; MAX_N] = [0; MAX_N];
        table[0] = 1;
        for i in 1..MAX_N {
            table[i] = i * table[i - 1];
        }
        table
    };

    // Determine the block_size to use. If n! is less than
    // PREFERRED_NUMBER_OF_BLOCKS_TO_USE then just use a single block to prevent
    // block_size from being set to 0. This also causes smaller values of n to
    // be computed serially which is faster and uses less resources for small
    // values of n.
    let block_size = 1.max(factorial_lookup_table[n] / PREFERRED_NUMBER_OF_BLOCKS_TO_USE);
    let block_count = factorial_lookup_table[n] / block_size;

    // Iterate over each block.
    (0..block_count)
        .into_par_iter()
        .map(|bn| {
            let initial_permutation_index = bn * block_size;

            let mut count: [usize; MAX_N] = [0; MAX_N];
            let mut current_permutation: [u8; MAX_N] =
                [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];

            // Initialize count and current_permutation.
            {
                let mut temp_permutation: [u8; MAX_N] = [0; MAX_N];
                let mut permutation_index = initial_permutation_index;
                for i in (1..n).rev() {
                    let f = factorial_lookup_table[i];
                    let d = permutation_index / f;

                    count[i] = d;

                    // Rotate the permutation left by d places. This is faster
                    // than using slice::rotate_left.
                    temp_permutation[0..=i - d]
                        .copy_from_slice(&current_permutation[d..=i]);
                    temp_permutation[i - d + 1..=i]
                        .copy_from_slice(&current_permutation[..d]);
                    current_permutation = temp_permutation;

                    permutation_index %= f;
                }
            }

            let mut max_flip_count = 0;
            let mut checksum = 0;

            // Iterate over each permutation in the block.
            let last_permutation_index = initial_permutation_index + block_size;
            for permutation_index in
                initial_permutation_index..last_permutation_index
            {
                // If the first value in the current_permutation is not 1 (0)
                // then we will need to do at least one flip for the
                // current_permutation.
                if current_permutation[0] > 0 {
                    // Make a copy of current_permutation[] to work on.
                    let mut temp_permutation = current_permutation;

                    let mut flip_count: usize = 1;

                    // Flip temp_permutation until the element at the
                    // first_value index is 1 (0).
                    let mut first_value = current_permutation[0] as usize & 0xF;
                    while temp_permutation[first_value] > 0 {
                        // Record the new_first_value and restore the old
                        // first_value at its new flipped position.
                        let new_first_value = replace(
                            &mut temp_permutation[first_value],
                            first_value as u8,
                        );

                        // If first_value is greater than 3 (2) then we are
                        // flipping a series of four or more values so we will
                        // also need to flip additional elements in the middle
                        // of the temp_permutation.
                        if first_value > 2 {
                            for (low_index, high_index) in
                                (1..first_value).zip((1..first_value).rev())
                            {
                                temp_permutation.swap(high_index, low_index);

                                if low_index + 3 > high_index {
                                    break;
                                }
                            }
                        }

                        // Update first_value to new_first_value that we
                        // recorded earlier.
                        first_value = new_first_value as usize & 0xF;
                        flip_count += 1;
                    }

                    // Update the checksum.
                    if permutation_index % 2 == 0 {
                        checksum += flip_count;
                    } else {
                        checksum -= flip_count;
                    }

                    // Update max_flip_count if necessary.
                    max_flip_count = max_flip_count.max(flip_count);
                }

                // Generate the next permutation.
                current_permutation.swap(0, 1);
                let mut first_value = current_permutation[0];
                for i in 1..MAX_N - 2 {
                    count[i] += 1;
                    if count[i] <= i {
                        break;
                    }
                    count[i] = 0;

                    let new_first_value = current_permutation[1];

                    for j in 0..i + 1 {
                        current_permutation[j] = current_permutation[j + 1];
                    }

                    current_permutation[i + 1] = first_value;
                    first_value = new_first_value;
                }
            }
            (checksum, max_flip_count)
        })
        .reduce(
            || (0, 0),
            |(cs1, mf1), (cs2, mf2)| (cs1 + cs2, mf1.max(mf2)),
        )
}


// An async function that consumes a request, does nothing with it and returns a
// response.
async fn rustless(req: Request<impl hyper::body::Body>) -> Result<Response<Full<Bytes>>, Infallible> {

   let query = req.uri().query();
    let n = query
        .and_then(|q| q.split('=').nth(1))
        .and_then(|val| val.parse::<usize>().ok())
        .unwrap_or(12);

    if n >= MAX_N {
        return Ok(Response::new(Full::new(Bytes::from(format!(
            "Value of n must be less than {}",
            MAX_N
        )))));
    }

    let (checksum, max_flip_count) = compute_fannkuch_redux(n);

    let response_body = format!(
            "{}\nPfannkuchen({}) = {}\nHello, Rustless!",
            checksum, n, max_flip_count
        );

    Ok(Response::new(Full::new(Bytes::from(response_body))))
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