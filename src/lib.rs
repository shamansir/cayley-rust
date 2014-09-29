#![crate_name = "cayley"]

extern crate debug;
extern crate http;
extern crate url;
extern crate serialize;

use std::str;
use std::io::println;
use std::io::Stream;
use std::slice::Items;
use url::Url;
use http::client::{RequestWriter, ResponseReader};
use http::method::{Get, Post};
use http::headers::HeaderEnum;
use serialize::{Decoder, Decodable};
use serialize::json::decode as json_decode;
use serialize::json::DecoderError;

use graph_error::{GraphRequestError, GraphResult,
                  InvalidUrl, MalformedRequest, RequestFailed, DecodingFailed, ResponseParseFailed};

mod graph_error;

pub struct Graph {
    url: String,
    path: Vec<String>, // FIXME: change to "Vec<u8>" or "Vec<&str>"?
    request: Box<RequestWriter>/*,
    use_ssl: bool*/
}

pub struct GraphNode {
    id: String
}

pub enum Selector {
    Specific(String),
    Every
}

pub enum CayleyAPIVersion { V1, DefaultVersion }

pub struct GraphAccess {
    pub host: &'static str,
    pub version: CayleyAPIVersion,
    pub port: int
}

pub struct GraphNodes(Vec<GraphNode>);

impl Graph {

    pub fn new(access: GraphAccess) -> GraphResult<Graph> {
        Graph::at(access.host, access.port, access.version)
    }

    pub fn default() -> GraphResult<Graph> {
        Graph::at("localhost", 64210, V1)
    }

    pub fn at(host: &str, port: int, version: CayleyAPIVersion) -> GraphResult<Graph> {
        let version_str = match version { V1 | DefaultVersion => "v1" };
        let url = format!("http://{:s}:{:d}/api/{:s}/query/gremlin",
                          host, port, version_str);
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

    fn make_request(url: &str) -> GraphResult<Box<RequestWriter>> {
        match Url::parse(url) {
            Err(error) => Err(InvalidUrl(error, url.to_string())),
            Ok(parsed_url) => {
                match RequestWriter::new(Post, parsed_url) {
                    Err(error) => Err(MalformedRequest(error, url.to_string())),
                    Ok(request) => Ok(box request)
                }
            }
        }
    }

    fn decode_nodes(source: Vec<u8>) -> GraphResult<GraphNodes> {
        match str::from_utf8(source.as_slice()) {
            None => Err(ResponseParseFailed),
            Some(nodes_json) => {
                match json_decode(nodes_json) {
                    Err(error) => Err(DecodingFailed(error, nodes_json.to_string())),
                    Ok(nodes) => Ok(nodes)
                }
            }
        }
    }

    fn exec_path(mut request: Box<RequestWriter>, path: &str) -> GraphResult<GraphNodes> {
        request.headers.content_length = Some(path.len());
        match request.write_str(path) {
            Err(error) => Err(RequestFailed(error, path.to_string())),
            Ok(_) => match request.read_response() {
                Err((_, error)) => Err(RequestFailed(error, path.to_string())),
                Ok(mut response) => match response.read_to_end() {
                    Err(error) => Err(RequestFailed(error, path.to_string())),
                    Ok(body) => Graph::decode_nodes(body)
                }
            }
        }
    }

    pub fn all(&self) -> GraphResult<GraphNodes> {
        self.path.push("All()".to_string());
        let full_path = self.path.connect(".");
        let nodes = try!(Graph::exec_path(self.request, full_path.as_slice()));
        self.path.clear();
        Ok(nodes)
    }

    pub fn get_limit(&self, limit: int) -> GraphResult<GraphNodes> {
        self.path.push(format!("GetLimit({:i})", limit));
        let full_path = self.path.connect(".");
        let nodes = try!(Graph::exec_path(self.request, full_path.as_slice()));
        self.path.clear();
        Ok(nodes)
    }

    pub fn v(&self, what: Selector) -> &Graph {
        match what {
            Every /*| Specific("")*/ => { self.path.push("Vertex()".to_string()); },
            Specific(name) => { self.path.push(format!("Vertex(\"{:s}\")", name)); }
        }
        self
    }

    pub fn vertex(&self, what: Selector) -> &Graph { self.v(what) }

    pub fn _in(&self, _where: &str) -> &Graph {
        self.path.push(format!("in(\"{:s}\")", _where));
        self
    }

}

impl GraphNode {

    pub fn id(self) -> String { self.id }

}

impl<S: Decoder<E>, E> Decodable<S, E> for GraphNode {
    fn decode(decoder: &mut S) -> Result<GraphNode, E> {
        decoder.read_struct("__unused__", 0, |decoder| {
            Ok(GraphNode {
                id: try!(decoder.read_struct_field("id", 0,
                        |decoder| { decoder.read_str() }))
            })
        })
    }
}

impl<S: Decoder<E>, E> Decodable<S, E> for GraphNodes {
    fn decode(decoder: &mut S) -> Result<GraphNodes, E> {
        decoder.read_struct("__unused__", 0, |decoder| {
            decoder.read_struct_field("result", 0, |decoder| {
                decoder.read_seq(|decoder, len| {
                    let mut nodes: Vec<GraphNode> = Vec::with_capacity(len);
                    for i in range(0u, len) {
                        nodes.push(match decoder.read_seq_elt(i,
                                        |decoder| { Decodable::decode(decoder) }) {
                            Ok(node) => node,
                            Err(err) => return Err(err)
                        });
                    };
                    Ok(GraphNodes(nodes))
                })
            })
        })
    }
}

pub fn make_and_print_request(url: &str, body: &str) {
    // echo "graph.Vertex('Humphrey Bogart').All()" |
    // http --verbose POST localhost:64210/api/v1/query/gremlin Content-Type:text/plain

    let url = Url::parse(url).ok().expect("Invalid URL :-(");
    let mut request: RequestWriter = RequestWriter::new(Post, url).unwrap();

    request.headers.content_length = Some(body.len());
    request.write_str(body);

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
    println!("[1mBody:[0m");
    println!("{}", body);

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
