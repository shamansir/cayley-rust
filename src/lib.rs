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

/* struct Path<'g> {
    value: Vec<&'g str>
}

impl<'g> Path<'g> {

    pub fn new() -> Path<'g> {
        let mut value = Vec::with_capacity(30);
        value.push("graph");
        Path{ value: value }
    }

    pub fn add<'g>(mut self, segment: &'g str) -> Path<'g> {
        self.value.push(segment);
        self
    }

} */

pub struct Graph<'g> {
    url: String, // FIXME: change to "&'g str"
    path: Vec<&'g str>,
    request: Box<RequestWriter>
}

pub enum NodeSpec {
    NodeName(&'static str),
    AnyNode
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

impl<'g> Graph<'g> {

    pub fn new(access: GraphAccess) -> Result<Graph<'g>, GraphRequestError> {
        Graph::at(access.host, access.port, access.version)
    }

    pub fn default() -> Result<Graph<'g>, GraphRequestError> {
        Graph::at("localhost", 64210, "v1")
    }

    pub fn at<'g>(host: &str, port: int, version: &str) -> Result<Graph<'g>, GraphRequestError> {
        let url = format!("https://{:s}:{:d}/api/{:s}/query/gremlin/",
                          host, port, version);
        match Graph::make_request(url.as_slice()) {
            Ok(request) => { let mut path = Vec::with_capacity(30);
                             path.push("graph");
                             Ok(Graph{ url: url,
                                       path: path,
                                       request: request }) },
            Err(error) => Err(error)
        }
    }

    fn make_request(url: &str) -> Result<Box<RequestWriter>, GraphRequestError> {
        match Url::parse(url.as_slice()) {
            Err(error) => Err(InvalidUrl(error)),
            Ok(parsed_url) => {
                match RequestWriter::new(Get, parsed_url) {
                    Err(error) => Err(MalformedRequest(error)),
                    Ok(request) => Ok(box request)
                }
            }
        }
    }

    pub fn v(mut self, what: NodeSpec) -> Graph<'g> {
        match what {
            AnyNode => { self.path.push("Vertex()"); },
            NodeName(name) => { self.path.push(concat!("Vertex(\"{", name, "\"}")); }
        }
        self
    }

    /* fn ask_cayley(&self, path: &str) -> Result<GraphNode, GraphRequestError> {
        let request = match self.check_connection() {
            Ok(ref value) => value,
            Err(e) => return Err(e)
        };
        Ok(GraphNode)
    } */
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
