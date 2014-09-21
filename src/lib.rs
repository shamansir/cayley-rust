#![crate_name = "cayley"]

extern crate debug;
extern crate http;
extern crate url;

use http::client::RequestWriter;
use http::method::Get;
use http::headers::HeaderEnum;
use std::str;
use std::io::println;
use std::io::{IoResult, IoError};
use std::fmt::{Show, Formatter, FormatError};
use url::{Url, ParseError};

/* pub enum CayleyAPIVersion { V1 } */

pub struct GraphAccess<'a> {
    pub host: &'a str,
    pub version: &'a str,
    pub port: int
}

pub struct Graph {
    url: &'static str,
    request: Option<Box<RequestWriter>>
}

pub struct GraphNode;

pub enum GraphRequestError {
    InvalidUrl(ParseError),
    MalformedRequest(IoError),
    RequestFailed(IoError)
}

impl Show for GraphRequestError {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), FormatError> {
        match *self {
            InvalidUrl(ref perr) => perr.fmt(fmt),
            MalformedRequest(ref ioerr) => ioerr.fmt(fmt),
            RequestFailed(ref ioerr) => ioerr.fmt(fmt)
        }
    }
}

impl Graph {

    pub fn new(access: Option<GraphAccess>) -> Graph {
        match access {
            Some(value) => Graph::at(value.host, value.port, value.version),
            None => Graph::at("localhost", 64210, "v1")
        }
    }

    pub fn at<'a>(host: &'a str, port: int, version: &'a str) -> Graph {
        Graph{ url: format!("https://{:s}:{:d}/api/{:s}/query/gremlin/",
                            host, port, version).as_slice(),
               request: None }
    }

    fn try_to_connect(&self) -> Result<Box<RequestWriter>, GraphRequestError> {
        let request_url = match Url::parse(self.url) {
            Ok(value) => value,
            Err(e) => return Err(InvalidUrl(e))
        };
        match RequestWriter::new(Get, request_url) {
            Ok(value) => Ok(box value),
            Err(e) => Err(MalformedRequest(e))
        }
    }

    fn make_request(&mut self, path: &str) -> Result<GraphNode, GraphRequestError> {
        let request: Box<RequestWriter> = match self.request {
            Some(value) => value,
            None => match self.try_to_connect() {
                        Ok(value) => value,
                        Err(e) => return Err(e)
                    }
        };
        self.request = Some(request);
        Ok(GraphNode)
    }
}

pub fn make_and_print_request(url: &str) {
    // echo "graph.Vertex('Humphrey Bogart').All()" |
    // http --verbose POST localhost:64210/api/v1/query/gremlin Content-Type:text/plain

    let url = Url::parse(url).ok().expect("Invalid URL :-(");
    let request: RequestWriter = RequestWriter::new(Get, url).unwrap();

    println!("[33;1mRequest[0m");
    println!("[33;1m=======[0m");
    println!("");
    println!("[1mURL:[0m {}", request.url);
    println!("[1mRemote address:[0m {}", request.remote_addr);
    println!("[1mMethod:[0m {}", request.method);
    println!("[1mHeaders:[0m");
    for header in request.headers.iter() {
        println!(" - {}: {}", header.header_name(), header.header_value());
    }

    println!("");
    println!("[33;1mResponse[0m");
    println!("[33;1m========[0m");
    println!("");
    let mut response = match request.read_response() {
        Ok(response) => response,
        Err(_request) => fail!("This example can progress no further with no response :-("),
    };
    println!("[1mStatus:[0m {}", response.status);
    println!("[1mHeaders:[0m");
    for header in response.headers.iter() {
        println!(" - {}: {}", header.header_name(), header.header_value());
    }
    println!("[1mBody:[0m");
    let body = match response.read_to_end() {
        Ok(body) => body,
        Err(err) => fail!("Reading response failed: {}", err),
    };
    println(str::from_utf8(body.as_slice()).expect("Uh oh, response wasn't UTF-8"));
}
