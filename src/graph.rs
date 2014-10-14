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

use std::collections::HashMap;

use path::Query;
use path::Reuse;

use errors::{GraphRequestError, GraphResult,
             InvalidUrl, MalformedRequest, RequestFailed,
             DecodingFailed, ResponseParseFailed,
             QueryNotFinalized, QueryCompilationFailed,
             ReusableCannotBeSaved };

pub struct Graph {
    url: String
}

pub struct GraphNode<'gn>(pub HashMap<&'gn str, &'gn str>);

pub struct GraphNodes<'gns>(pub Vec<GraphNode<'gn>>);

pub enum CayleyAPIVersion { V1, DefaultVersion }

impl Graph {

    pub fn default() -> GraphResult<Graph> {
        Graph::new("localhost", 64210, DefaultVersion)
    }

    pub fn new(host: &str, port: int, version: CayleyAPIVersion) -> GraphResult<Graph> {
        let version_str = match version { V1 | DefaultVersion => "v1" };
        let url = format!("http://{:s}:{:d}/api/{:s}/query/gremlin",
                          host, port, version_str);
        Ok(Graph{ url: url })
    }

    // find nodes by query implementation and return them parsed
    pub fn find(&self, query: &Query) -> GraphResult<GraphNodes> {
        if query.is_finalized() {
            match query.compile() {
                Some(compiled) => self.find_by(compiled),
                None => Err(QueryCompilationFailed)
            }
        } else { Err(QueryNotFinalized) }
    }

    // find nodes using raw pre-compiled string query and return them parsed
    pub fn find_by(&self, query: String) -> GraphResult<GraphNodes> {
        match self.perform_request(query) {
            Ok(body) => Graph::decode_nodes(body),
            Err(error) => Err(error)
        }
    }

    pub fn save(&self, reusable: &mut Reuse) -> GraphResult<()> {
        match reusable.save() {
            Some(query) => {
                match self.perform_request(query) {
                    Ok(body) => { reusable.set_saved(); Ok(()) },
                    Err(error) => Err(error)
                }
            },
            None => Err(ReusableCannotBeSaved)
        }
    }

    pub fn save_as(&self, name: &str, reusable: &mut Reuse) -> GraphResult<()> {
        match reusable.save_as(name) {
            Some(query) => {
                match self.perform_request(query) {
                    Ok(body) => { reusable.set_saved(); Ok(()) },
                    Err(error) => Err(error)
                }
            },
            None => Err(ReusableCannotBeSaved)
        }
    }

    // uses RequestWriter to perform a request with given request body and returns the response body
    fn perform_request(&self, body: String) -> GraphResult<Vec<u8>> {
        match Graph::prepare_request(self.url.as_slice()) {
            Err(error) => Err(error),
            Ok(mut request) => {
                request.headers.content_length = Some(body.len());
                match request.write_str(body.as_slice()) {
                    Err(error) => Err(RequestFailed(error, body)),
                    Ok(_) => match request.read_response() {
                        Err((_, error)) => Err(RequestFailed(error, body)),
                        Ok(mut response) => match response.read_to_end() {
                            Err(error) => Err(RequestFailed(error, body)),
                            Ok(response_body) => Ok(response_body)
                        }
                    }
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

impl<S: Decoder<E>, E> Decodable<S, E> for GraphNode {
    fn decode(decoder: &mut S) -> Result<GraphNode, E> {
        decoder.read_map(|decoder, len| {
            let mut data_map: HashMap<String, String> = HashMap::new();
            for i in range(0u, len) {
                data_map.insert(match decoder.read_map_elt_key(i, |decoder| { decoder.read_str() }) {
                                    Ok(key) => key, Err(err) => return Err(err)
                                },
                                match decoder.read_map_elt_val(i, |decoder| { decoder.read_str() }) {
                                    Ok(val) => val, Err(err) => return Err(err)
                                });
            }
            Ok(GraphNode(data_map))
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
