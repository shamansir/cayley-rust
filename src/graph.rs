use std::str;
use std::io::println;
use std::io::Stream;
use url::Url;
use http::client::RequestWriter;
use http::method::Post;
use http::headers::HeaderEnum;
use serialize::{Decoder, Decodable};
use serialize::json::decode as json_decode;
use serialize::json::DecoderError;

use path::Query;

use errors::{GraphRequestError, GraphResult,
             InvalidUrl, MalformedRequest, RequestFailed,
             DecodingFailed, ResponseParseFailed,
             QueryNotFinalized, QueryCompilationFailed };

pub struct Graph {
    url: String,
    request: Box<RequestWriter>
}

pub struct GraphNode {
    id: String
}

pub struct GraphNodes(pub Vec<GraphNode>);

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