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
                  InvalidUrl, MalformedRequest, RequestFailed,
                  DecodingFailed, ResponseParseFailed,
                  QueryCompilationFailed};

mod graph_error;

pub struct Graph {
    url: String,
    request: Box<RequestWriter>
}

pub struct GraphNode {
    id: String
}

pub struct GraphNodes(pub Vec<GraphNode>);

pub struct Vertex {
    ready: bool,
    path: Vec<String>
}

pub struct Morphism<'m> {
    query: Box<Query+'m>
}

pub enum NodeSelector<'ns> {
    EveryNode,
    Node(&'ns str),
    Nodes(Vec<&'ns str>)
}

pub enum PredicateSelector<'m> {
    EveryPredicate,
    Predicate(&'m str),
    Predicates(Vec<&'m str>),
    FromQuery(Box<Query+'m>)
}

pub enum TagSelector<'ts> {
    EveryTag,
    Tag(&'ts str),
    Tags(Vec<&'ts str>)
}

pub enum CayleyAPIVersion { V1, DefaultVersion }

impl Graph {

    pub fn default() -> GraphResult<Graph> {
        Graph::new("localhost", 64210, DefaultVersion)
    }

    pub fn new(host: &str, port: int, version: CayleyAPIVersion) -> GraphResult<Graph> {
        let version_str = match version { V1 | DefaultVersion => "v1" };
        let url = format!("http://{:s}:{:d}/api/{:s}/query/gremlin",
                          host, port, version_str);
        match Graph::prepare_request(url.as_slice()) {
            Ok(request) => { // TODO: match request.try_connect()
                             let mut path: Vec<String> = Vec::with_capacity(20);
                             path.push("graph".to_string());
                             Ok(Graph{ url: url,
                                       request: request }) },
            Err(error) => Err(error)
        }
    }

    pub fn find_by(mut self, query: String) -> GraphResult<GraphNodes> {
        let mut request = self.request;
        request.headers.content_length = Some(query.len());
        match request.write_str(query.as_slice()) {
            Err(error) => Err(RequestFailed(error, query)),
            Ok(_) => match request.read_response() {
                Err((_, error)) => Err(RequestFailed(error, query)),
                Ok(mut response) => match response.read_to_end() {
                    Err(error) => Err(RequestFailed(error, query)),
                    Ok(body) => Graph::decode_nodes(body)
                }
            }
        }
    }

    pub fn find(mut self, query: &Query) -> GraphResult<GraphNodes> {
        match query.compile() {
            Some(compiled) => self.find_by(compiled),
            None => Err(QueryCompilationFailed)
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

    fn prepare_request(url: &str) -> GraphResult<Box<RequestWriter>> {
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

}

pub trait AddString {

    fn add_str(&mut self, str: &str) -> &Self;

    fn add_string(&mut self, str: String) -> &Self;

}

// FIXME: may conflict with std::Path
pub trait Path: AddString/*+ToString*/ {

    fn compile(&self) -> Option<String>;

    /* fn to_string(&self) -> String {
        match self.compile() {
            Some(compiled) => compiled,
            None => "[-]".to_string()
        }
    }*/

    fn out(&mut self, predicates: PredicateSelector, tags: TagSelector) -> &Self {
        self.add_string(format!("Out({:s})", make_args_from(predicates, tags)))
    }

    // TODO: in, both...

}

pub trait Query: Path {

    pub fn all(&mut self) -> &Vertex { self.ready = true; self.add_str("All()") }

    // TODO: get_limit....

}

impl Vertex {

    fn start(nodes: NodeSelector) -> Vertex {
        let mut res = Vertex{ path: Vec::with_capacity(10), ready: false };
        res.add_str("graph");
        res.add_string(match nodes {
                EveryNode/*| Node("") */ => "Vertex()".to_string(),
                Node(name) => format!("Vertex(\"{:s}\")", name),
                Nodes(names) => format!("Vertex(\"{:s}\")", names.connect(","))
            });
        res
    }

    /* fn all(self) -> Vertex {
        self.add("all".to_string());
        self
    } */

}

impl AddString for Vertex {

    fn add_str(&mut self, str: &str) -> &Vertex {
        self.path.push(str.to_string());
        self
    }

    fn add_string(&mut self, str: String) -> &Vertex {
        self.path.push(str);
        self
    }

}

impl Path for Vertex { }

impl Query for Vertex {

    fn compile(&self) -> Option<String> {
        match self.ready {
            true => Some(self.path.connect(".")),
            false => None
        }
    }

    // pub fn all(&mut self) -> &Vertex;

}

// TODO: impl Path for Morphism

/* impl Vertex {

    fn compile(&self) -> String {
        let VertexPath(path) = self;
        path.connect(".");
    }

    pub fn all(&self) -> GraphResult<GraphNodes> {

        path.push("All()".to_string());
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

} */

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


fn make_args_from(predicates: PredicateSelector, tags: TagSelector) -> String {
    match (predicates, tags) {
        (EveryPredicate, EveryTag) => "".to_string(),
        (EveryPredicate, Tag(tag)) => format!("null, \"{:s}\"", tag),
        (EveryPredicate, Tags(tags)) => format!("null, \"{:s}\"", tags.connect("\",\"")),
        (Predicate(predicate), EveryTag) => format!("\"{:s}\"", predicate),
        (Predicate(predicate), Tag(tag)) =>
            format!("\"{:s}\", \"{:s}\"", predicate, tag),
        (Predicate(predicate), Tags(tags)) =>
            format!("\"{:s}\", \"{:s}\"", predicate, tags.connect("\",\"")),
        (Predicates(predicates), EveryTag) =>
            format!("\"{:s}\"", predicates.connect("\",\"")),
        (Predicates(predicates), Tag(tag)) =>
            format!("\"{:s}\", \"{:s}\"", predicates.connect("\",\""), tag),
        (Predicates(predicates), Tags(tags)) =>
            format!("\"{:s}\", \"{:s}\"", predicates.connect("\",\""), tags.connect("\",\"")),
        (FromQuery(query), EveryTag) =>
            match query.compile() {
                Some(compiled) => compiled,
                None => "undefined".to_string()
            },
        (FromQuery(query), Tag(tag)) =>
            format!("{:s}, \"{:s}\"",
                    match query.compile() {
                        Some(compiled) => compiled,
                        None => "undefined".to_string()
                    },
                    tag),
        (FromQuery(query), Tags(tags)) =>
            format!("{:s}, \"{:s}\"",
                    match query.compile() {
                        Some(compiled) => compiled,
                        None => "undefined".to_string()
                    },
                    tags.connect("\",\"")),
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
