extern crate rustc_serialize;

use std::str;
use std::io::{Write, Read};

use self::rustc_serialize::{Decoder, Decodable};
use self::rustc_serialize::json::decode as json_decode;

use std::collections::HashMap;

use hyper::Url;
use hyper::client::Request;
use hyper::method::Method;
use hyper::header::ContentLength;

use path::CompiledQuery;

use path::Expectation;
use path::Expectation::{ SingleNode, SingleTag,
                         NameSequence, TagSequence };

use url::ParseError;

use error::Result as GraphResult;
use error::Error::{ InvalidUrl, MalformedRequest, RequestIoFailed, RequestFailed,
                    DecodingFailed, ResponseParseFailed, ExpectationNotSupported };

/// Provides access to currently running Cayley database, among with
/// an ability to run queries there, and to write there your data
/// (honestly, only if there's a `graph.emit()` method below—if not,
/// it will just soon be there).
///
/// * Use `Graph::default()` to connect to `localhost:64210`.
/// * Use `Graph::new(host, port, api_version)` to specify the location of database manually.
///
/// * Use `Graph::find(<Query>)` to find anything using [Query](../path/trait.Query.html) trait implementor
/// (`Query`, for example, is implemented by [Vertex](../path/struct.Vertex.html)), which in its turn
/// is similar to [Gremlin API](https://github.com/google/cayley/blob/master/docs/GremlinAPI.md).
/// * Use `Graph::find_by(<String>)` to find anything using [Gremlin API](https://github.com/google/cayley/blob/master/docs/GremlinAPI.md) API
/// from a prepared string. A raw, but not so beautiful, way to execute query.
/// * Use `Graph::save(<Path>)` to save a [Morphism](../path/struct.Morphism.html).
pub struct Graph {
    url: String
}

/// A wrapper for a single item Cayley returned in response for a query

/// This is a subject to change, since I'd prefer here would be `&str`
/// items inside, but it's quite hard to achieve this with `json::Decoder`
/* pub enum QueryObject {
    SingleNode(HashMap<String, String>), // Query.ToValue()
    NodeSequence(Vec<HashMap<String, String>>), // Query.All(), Query.GetLimit(n)
    NameSequence(Vec<String>), // Query.ToArray()
    TagSequence(Vec<String>), // Query.TagArray()
    SingleTag(String) // Query.TagValue()
} */

pub struct Nodes(pub Vec<HashMap<String, String>>);

/// Cayley API Version, planned to default to the latest, if it will ever change
pub enum APIVersion { V1, DefaultVersion }

impl Graph {

    // ---------------------------------- default ------------------------------

    /// Create a Graph which connects to the latest API at `localhost:64210`
    pub fn default() -> GraphResult<Graph> {
        Graph::new("localhost", 64210, APIVersion::DefaultVersion)
    }

    // ---------------------------------- new ----------------------------------

    /// Create a Graph which connects to the host you specified manually
    pub fn new(host: &str, port: i32, version: APIVersion) -> GraphResult<Graph> {
        let version_str = match version {
            APIVersion::V1 | APIVersion::DefaultVersion => "v1" /* FIXME: APIVersion:: shouldn't be required */
        };
        let url = format!("http://{host}:{port}/api/{version}/query/gremlin",
                          host = host, port = port, version = version_str);
        Ok(Graph{ url: url })
    }

    // ---------------------------------- find ---------------------------------

    /// Find nodes with the Query implementation (say, Vertex-path) and return them parsed
    ///
    /// Since only [Vertex](../path/struct.Vertex.html) implements [Query](../path/trait.Query.html) trait
    /// following current spec, your code will look like that:
    ///
    /// ```ignore
    /// use cayley::graph::Graph;
    /// use cayley::path::{Vertex, Path, Query};
    /// use cayley::selector::{Predicate, Node};
    ///
    /// let graph = Graph::default().unwrap();
    /// graph.find(Vertex::start(Node("foo")).InP(Predicate("bar")).All()).unwrap();
    /// ```
    pub fn find(&self, query: CompiledQuery) -> GraphResult<Nodes> {
        self.exec(query.prefix + &query.value, query.expectation)
    }

    // ---------------------------------- exec ---------------------------------

    /// Find nodes using raw pre-compiled query string and return them parsed
    ///
    /// If you want to run just the pure stringified Gremlin queries, bypassing
    /// the string concatenation performed with `path::` module members, this
    /// method is for you.
    ///
    /// ```ignore
    /// use cayley::Graph;
    /// let graph = Graph::default().unwrap();
    /// graph.exec("g.V(\"foo\").In(\"bar\").All()".to_string()).unwrap();
    /// ```
    pub fn exec(&self, query: String, expectation: Expectation) -> GraphResult<Nodes> {
        debug!("Executing query: {}", query);
        match expectation {
            SingleNode | NameSequence | TagSequence | SingleTag =>
                Err(ExpectationNotSupported(expectation)),
            _ => match self.perform_request(query.into_bytes()) {
                Ok(body) => Graph::decode_traversal(body, expectation),
                Err(error) => Err(error)
            }
        }
    }

    fn perform_request(&self, body: Vec<u8>) -> GraphResult<Vec<u8>> {
        let mut request = {
            let url_str = self.url.as_str();
            match Url::parse(url_str) {
                Err(error) => return Err(InvalidUrl(error, url_str.to_string())),
                Ok(parsed_url) => match Request::new(Method::Post, parsed_url) {
                    Err(error) => return Err(MalformedRequest(error, url_str.to_string())),
                    Ok(request) => request
                }
            }
        };
        request.headers_mut().set(ContentLength(body.len() as u64));
        match request.start() {
            Err(error) => return Err(RequestFailed(error, body)),
            Ok(mut request) => match request.write(body.as_slice()) {
                Err(error) => return Err(RequestIoFailed(error, body)),
                Ok(_) => match request.send() {
                    Err(error) => return Err(RequestFailed(error, body)),
                    Ok(mut response) => {
                        let mut response_body = Vec::new();
                        match response.read_to_end(&mut response_body) {
                            Err(error) => Err(RequestIoFailed(error, body)),
                            Ok(_) => {
                                debug!("Request to {} succeeded", self.url);
                                Ok(response_body)
                            }
                        }
                    }
                }
            }
        }

    }

    // extract JSON nodes from response
    #[allow(unused_variables)]
    fn decode_traversal(source: Vec<u8>, expectation: Expectation) -> GraphResult<Nodes> {
        match str::from_utf8(source.as_slice()) {
            Err(_) => Err(ResponseParseFailed),
            Ok(traversal_json) => {
                debug!("start decoding \n===\n{:.200}\n...\n===\n", traversal_json);
                match json_decode(traversal_json) {
                    Err(error) => Err(DecodingFailed(error, traversal_json.to_string())),
                    Ok(nodes) => {
                        debug!("Returned: {}", match nodes { Nodes(ref val) => val.len() });
                        Ok(nodes)
                    }
                }
            }
        }
    }

}

impl Decodable for Nodes {

    fn decode<D: Decoder>(decoder: &mut D) -> Result<Self, D::Error> {
        decode_nodes(decoder)
    }
}

fn decode_nodes<D: Decoder>(decoder: &mut D) -> Result<Nodes, D::Error> {
    decoder.read_struct("__unused__", 0, |decoder| {
        match decoder.read_struct_field("error", 0, |d| -> Result<Option<String>, D::Error> { Decodable::decode(d) }) {
            Ok(val) => {
                match val {
                    Some(ref explanation) =>
                        Err(decoder.error(format!("Error returned from request: {}", explanation).as_str())),
                    None => decoder.read_struct_field("result", 1, |decoder| {
                        decoder.read_option(|decoder, has_value| {
                            match has_value {
                                false => Ok(Nodes(Vec::new())),
                                true => decoder.read_seq(|decoder, len| {
                                    let mut nodes: Vec<HashMap<String, String>> = Vec::with_capacity(len);
                                    for i in 0..len {
                                        nodes.push(match decoder.read_seq_elt(i,
                                            |decoder| { decode_node(decoder) }) {
                                                Ok(node) => node,
                                                Err(err) => return Err(err)
                                            });
                                        };
                                    Ok(Nodes(nodes))
                                })
                            }
                        })
                    })
                }
            },
            Err(err) => { println!("err branch"); Err(err) }
        }
    })
}

fn decode_node<D: Decoder>(decoder: &mut D) -> Result<HashMap<String, String>, D::Error> {
    decoder.read_map(|decoder, len| {
        let mut data_map: HashMap<String, String> = HashMap::new();
        for i in 0..len {
            data_map.insert(match decoder.read_map_elt_key(i, |decoder| { decoder.read_str() }) {
                Ok(key) => key, Err(err) => return Err(err)
                },
                match decoder.read_map_elt_val(i, |decoder| { decoder.read_str() }) {
                    Ok(val) => val, Err(err) => return Err(err)
                });
        }
        Ok(data_map)
    })
}
