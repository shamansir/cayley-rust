#![crate_name = "cayley"]

extern crate debug;
extern crate http;
extern crate url;
extern crate serialize;

use std::str;
use std::io::println;
use url::Url;
use http::client::RequestWriter;
use http::method::Post;
use http::headers::HeaderEnum;
use serialize::{Decoder, Decodable};
use serialize::json::decode as json_decode;
//use serialize::json::DecoderError;

use graph_error::{GraphRequestError, GraphResult,
                  InvalidUrl, MalformedRequest, RequestFailed,
                  DecodingFailed, ResponseParseFailed,
                  QueryCompilationFailed, QueryNotFinalized};

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
    finalized: bool,
    path: Vec<String>
}

pub struct Morphism {
    name: String,
    path: Vec<String>
}

pub enum NodeSelector<'ns> {
    AnyNode,
    Node(&'ns str),
    Nodes(Vec<&'ns str>)
}

pub enum PredicateSelector<'m> {
    AnyPredicate,
    Predicate(&'m str),
    Predicates(Vec<&'m str>),
    FromQuery(Box<Query+'m>)
}

pub enum TagSelector<'ts> {
    AnyTag,
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

    // find nodes by query implementation
    pub fn find(self, query: &Query) -> GraphResult<GraphNodes> {
        match query.is_finalized() {
            true => match query.compile() {
                Some(compiled) => self.find_by(compiled),
                None => Err(QueryCompilationFailed)
            },
            false => Err(QueryNotFinalized)
        }
    }

    // find nodes using raw pre-compiled string
    pub fn find_by(self, query: String) -> GraphResult<GraphNodes> {
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

    // prepares the RequestWriter object from URL to save it inside the Graph for future re-use
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

    // extract JSON nodes from response
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

}

pub trait AddString {

    fn add_str(&mut self, str: &str) -> &Self;

    fn add_string(&mut self, str: String) -> &Self;

}

// FIXME: may conflict with std::Path
#[allow(non_snake_case)]
pub trait Path: AddString/*+ToString*/ {

    fn compile(&self) -> Option<String>;

    /* fn to_string(&self) -> String {
        match self.compile() {
            Some(compiled) => compiled,
            None => "[-]".to_string()
        }
    }*/

    fn Out(&mut self, predicates: PredicateSelector, tags: TagSelector) -> &Self {
        self.add_string(format!("Out({:s})", make_args_from(predicates, tags)))
    }

    // TODO: in, both...

}

#[allow(non_snake_case)]
pub trait Query: Path {

    fn set_finalized(&mut self);

    fn is_finalized(&self) -> bool;

    fn All(&mut self) -> &Self { self.set_finalized(); self.add_str("All()") }

    // TODO: get_limit....

}

impl Vertex {

    pub fn start(nodes: NodeSelector) -> Vertex {
        let mut res = Vertex{ path: Vec::with_capacity(10), finalized: false };
        res.add_str("graph");
        res.add_string(match nodes {
                Nodes(names) => format!("Vertex(\"{:s}\")", names.connect(",")),
                Node(name) => format!("Vertex(\"{:s}\")", name),
                AnyNode/*| Node("") */ => "Vertex()".to_string()
            });
        res
    }

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

impl Path for Vertex {

    fn compile(&self) -> Option<String> {
        // a bolt-hole to return None, if path was incorrectly constructed
        Some(self.path.connect("."))
    }

}

impl Query for Vertex {

    fn set_finalized(&mut self) { self.finalized = true; }

    fn is_finalized(&self) -> bool { self.finalized }

}

impl Morphism {

    pub fn start(name: &str) -> Morphism {
        let mut res = Morphism { name: name.to_string(), path: Vec::with_capacity(10) };
        res.add_string(name.to_string() + " = graph.Morphism()".to_string());
        res
    }

}

impl AddString for Morphism {

    fn add_str(&mut self, str: &str) -> &Morphism {
        self.path.push(str.to_string());
        self
    }

    fn add_string(&mut self, str: String) -> &Morphism {
        self.path.push(str);
        self
    }

}

impl Path for Morphism {

    fn compile(&self) -> Option<String> {
        // a bolt-hole to return None, if path was incorrectly constructed
        Some(self.path.connect("."))
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

fn make_args_from(predicates: PredicateSelector, tags: TagSelector) -> String {
    match (predicates, tags) {

        (AnyPredicate, AnyTag) => "".to_string(),
        (AnyPredicate, Tag(tag)) => format!("null, \"{:s}\"", tag),
        (AnyPredicate, Tags(tags)) => format!("null, \"{:s}\"", tags.connect("\",\"")),

        (Predicate(predicate), AnyTag) => format!("\"{:s}\"", predicate),
        (Predicate(predicate), Tag(tag)) =>
            format!("\"{:s}\", \"{:s}\"", predicate, tag),
        (Predicate(predicate), Tags(tags)) =>
            format!("\"{:s}\", \"{:s}\"", predicate, tags.connect("\",\"")),

        (Predicates(predicates), AnyTag) =>
            format!("\"{:s}\"", predicates.connect("\",\"")),
        (Predicates(predicates), Tag(tag)) =>
            format!("\"{:s}\", \"{:s}\"", predicates.connect("\",\""), tag),
        (Predicates(predicates), Tags(tags)) =>
            format!("\"{:s}\", \"{:s}\"", predicates.connect("\",\""), tags.connect("\",\"")),

        (FromQuery(query), AnyTag) =>
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
                    tags.connect("\",\""))

    }
}


// echo "graph.Vertex('Humphrey Bogart').All()" |
// http --verbose POST localhost:64210/api/v1/query/gremlin Content-Type:text/plain
