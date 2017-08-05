# path_router
Routing for paths delimited by a forward slash, with the ability to capture specified path segments.

It has only a single dependency on the [log](https://crates.io/crates/log) facade (which is actually "optional" too).

# Example
Routing for the [Hyper](https://crates.io/crates/hyper) server:

```rust
extern crate hyper;
extern crate futures;
extern crate path_router;

use hyper::server::{Http, Request, Response};
use hyper::header::ContentLength;
use futures::future::{FutureResult, ok};
use std::sync::Arc;

type Handler = fn(Request, Vec<(&str, String)>) -> Response;

fn main() {
    struct WebService<'a, T> {
        routes: Arc<path_router::Tree<'a, T>>
    }
    impl<'a, F> hyper::server::Service for WebService<'a, F>
        where F: Fn(Request, Vec<(&'a str, String)>) -> Response
        {
            type Request = Request;
            type Response = Response;
            type Error = hyper::Error;
            type Future = FutureResult<Self::Response, Self::Error>;
            fn call(&self, req: Request) -> Self::Future {
                let route = format!("{}{}", req.method(), req.uri().path());
                match self.routes.find(&route) {
                    Some((handler, captures)) => ok(handler(req, captures)),
                    None => ok(Response::new()
                               .with_status(hyper::StatusCode::NotFound))
                }
            }
        }

    let mut routes: path_router::Tree<Handler> = path_router::Tree::new();
    routes.add("GET/echo/:text", echo_handler);
    routes.add("GET/reverse/:text", reverse_handler);
    let routes_r = Arc::new(routes);

    let addr = "127.0.0.1:3000".parse().unwrap();
    let server = Http::new().bind(&addr, move || {
        Ok(WebService {
            routes: routes_r.clone()
        })
    }).unwrap();
    server.run().unwrap();
}

fn echo_handler(_req: Request, captures: Vec<(&str, String)>) -> Response {
    let text = captures.iter().find(|c| c.0 == "text").unwrap().1.to_owned();
    Response::new()
        .with_header(ContentLength(text.len() as u64))
        .with_body(text)
}

fn reverse_handler(_req: Request, captures: Vec<(&str, String)>) -> Response {
    let text = captures.iter().find(|c| c.0 == "text").unwrap().1.to_owned();
    let reversed: String = text.chars().rev().collect();
    Response::new()
        .with_header(ContentLength(text.len() as u64))
        .with_body(reversed)
}
```
