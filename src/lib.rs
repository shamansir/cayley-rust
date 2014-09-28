#![crate_name = "cayley"]

extern crate debug;
extern crate http;
extern crate url;
extern crate serialize;

use http::client::{RequestWriter, ResponseReader};
use http::method::Get;
use http::headers::HeaderEnum;
use std::str;
use std::io::println;
use std::io::Stream;
use url::Url;
use serialize::Decoder;
use serialize::json::DecoderError;

/* pub enum CayleyAPIVersion { V1 } */

use graph_error::{GraphRequestError,
                  InvalidUrl, MalformedRequest, RequestFailed, DecodingFailed, ResponseParseFailed};

mod graph_error;

pub struct Graph {
    url: String,
    path: Vec<String>, // FIXME: change to "Vec<u8>" or "Vec<&str>"?
    request: Box<RequestWriter>/*,
    use_ssl: bool*/
}

pub enum Selector {
    Specific(String),
    Every
}

pub struct GraphNode {
    value: &'static str
}

pub struct GraphNodes {
    nodes: Vec<GraphNode>
}

pub struct GraphAccess {
    pub host: &'static str,
    pub version: &'static str, // FIXME: should be auto-set
    pub port: int
}

impl GraphNodes {
    pub fn new() -> GraphNodes {
        GraphNodes {
            nodes: Vec::new()
        }
    }
}

impl Collection for GraphNodes {

    fn len(&self) -> uint { self.nodes.len() }

    fn is_empty(&self) -> bool { self.nodes.is_empty() }

}

impl Graph {

    pub fn new(access: GraphAccess) -> Result<Graph, GraphRequestError> {
        Graph::at(access.host, access.port, access.version)
    }

    pub fn default() -> Result<Graph, GraphRequestError> {
        Graph::at("localhost", 64210, "v1")
    }

    pub fn at(host: &str, port: int, version: &str) -> Result<Graph, GraphRequestError> {
        let url = format!("http://{:s}:{:d}/api/{:s}/query/gremlin/",
                          host, port, version);
        match Graph::make_request(url.as_slice()) {
            Ok(request) => { // TODO: match request.try_connect()
                             let mut path: Vec<String> = Vec::with_capacity(20);
                             path.push("graph".to_string());
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

    fn decode_nodes<S: Stream>(mut response: ResponseReader<S>) -> Result<GraphNodes, GraphRequestError> {
        match response.read_to_end() {
            Err(error) => Err(RequestFailed(error)),
            Ok(body) => {
                match str::from_utf8(body.as_slice()) {
                    None => Err(ResponseParseFailed),
                    Some(json) => {
                        /*match Graph::decode_nodes(json) {
                            Err(error) => Err(DecodingFailed(error)),
                            Ok(nodes) => Ok(nodes)
                        }*/
                        Ok(GraphNodes::new())
                    }
                }
            }
        }
    }

    pub fn all(mut self) -> Result<GraphNodes, GraphRequestError> {
        // TODO: convert to try! sequence
        let path = self.path.connect(".");
        self.path.clear();
        match self.request.write_str(path.as_slice()) {
            Err(error) => Err(RequestFailed(error)),
            Ok(_) => {
                match self.request.read_response() {
                    Err((_, error)) => Err(RequestFailed(error)),
                    Ok(response) => Graph::decode_nodes(response)
                }
            }
        }
    }

    pub fn v(mut self, what: Selector) -> Graph {
        match what {
            Every /*| Specific("")*/ => { self.path.push("Vertex()".to_string()); },
            Specific(name) => { self.path.push(format!("Vertex(\"{:s}\"", name)); }
        }
        self
    }

    pub fn vertex(self, what: Selector) -> Graph { self.v(what) }

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
