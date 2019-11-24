#[macro_use]
extern crate hyper;
extern crate futures;
extern crate rand;
extern crate tokio;

use futures::{future, future::Future, Stream};
use hyper::{service::service_fn, Body, Method, Request, Response, Server, StatusCode};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use std::{
    fs,
    io::{Error, ErrorKind},
    path::Path,
};
use tokio::fs::File;

static INDEX: &[u8] = b"Index to images service.";

fn main() {
    // Directory to hold images
    let files = Path::new("./files");
    fs::create_dir(files).ok();

    // Address
    let addr = ([127, 0, 0, 1], 8080).into();

    // Server builder
    let builder = Server::bind(&addr);

    // Request handling
    let server = builder.serve(move || service_fn(move |req| imageservice_handler(req, &files)));

    // Dealing with errors
    let server = server.map_err(drop);

    // Running the service
    hyper::rt::run(server);
}

// imageservice handler
fn imageservice_handler(
    req: Request<Body>,
    files: &Path,
) -> Box<dyn Future<Item = Response<Body>, Error = std::io::Error> + Send> {
    match (req.method(), req.uri().path().to_owned().as_ref()) {
        (&Method::GET, "/") => Box::new(future::ok(Response::new(INDEX.into()))),
        (&Method::POST, "/upload") => {
            let name: String = thread_rng().sample_iter(&Alphanumeric).take(20).collect();

            let mut filepath = files.to_path_buf();
            filepath.push(&name);

            let create_file = File::create(filepath);

            let write = create_file.and_then(|file| {
                req.into_body().map_err(other).fold(file, |file, chunk| {
                    tokio::io::write_all(file, chunk).map(|(file, _)| file)
                })
            });

            let body = write.map(|_| Response::new(name.into()));
            Box::new(body)
        }
        _ => response_with_code(StatusCode::NOT_FOUND),
    }
}

// response_with_code
fn response_with_code(
    status_code: StatusCode,
) -> Box<dyn Future<Item = Response<Body>, Error = Error> + Send> {
    let resp = Response::builder()
        .status(status_code)
        .body(Body::empty())
        .unwrap();

    Box::new(future::ok(resp))
}

// converting hyper::Error to io::Error
fn other<E>(err: E) -> Error
where
    E: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    Error::new(ErrorKind::Other, err)
}
