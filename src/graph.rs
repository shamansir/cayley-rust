extern crate log;

use std::str;

use serialize::{Decoder, Decodable};
use serialize::json::decode as json_decode;

use std::collections::HashMap;

use hyper::Url;
use hyper::client::Request;
use hyper::header::common::ContentLength;

use path::{Query, Path};

use path::Expectation;
use path::Expectation::{ Unknown,
                         SingleNode, SingleTag,
                         NodeSequence, NameSequence, TagSequence };

use errors::GraphResult;
use errors::RequestError::{ InvalidUrl, MalformedRequest, RequestIoFailed, RequestFailed,
                            DecodingFailed, ResponseParseFailed,
                            QueryNotFinalized, QueryCompilationFailed,
                            ExpectationNotSupported, VagueExpectation };

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
    pub fn new(host: &str, port: int, version: APIVersion) -> GraphResult<Graph> {
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
    /// ```
    /// use cayley::graph::Graph;
    /// use cayley::path::{Vertex, Path, Query};
    /// use cayley::selector::{Predicate, Node};
    ///
    /// let graph = Graph::default().unwrap();
    /// graph.find(Vertex::start(Node("foo")).InP(Predicate("bar")).All()).unwrap();
    /// ```
    pub fn find(&self, query: &Query) -> GraphResult<Nodes> {
        match query.compile() {
            Some((compiled, expectation)) => self.exec(compiled, expectation),
            None => Err(QueryCompilationFailed)
        }
    }

    // ---------------------------------- exec ---------------------------------

    /// Find nodes using raw pre-compiled query string and return them parsed
    ///
    /// If you want to run just the pure stringified Gremlin queries, bypassing
    /// the string concatenation performed with `path::` module members, this
    /// method is for you.
    ///
    /// ```
    /// use cayley::Graph;
    /// let graph = Graph::default().unwrap();
    /// graph.exec("g.V(\"foo\").In(\"bar\").All()".to_string()).unwrap();
    /// ```
    pub fn exec(&self, query: String, expectation: Expectation) -> GraphResult<Nodes> {
        debug!("Executing query: {}", query);
        match expectation {
            SingleNode | NameSequence | TagSequence | SingleTag =>
                Err(ExpectationNotSupported(expectation)),
            _ => match self.perform_request(query) {
                Ok(body) => Graph::decode_traversal(body, expectation),
                Err(error) => Err(error)
            }
        }
    }

    fn perform_request(&self, body: String) -> GraphResult<Vec<u8>> {
        let mut request = {
            let url_str = self.url.as_slice();
            match Url::parse(url_str) {
                Err(error) => return Err(InvalidUrl(error, url_str.to_string())),
                Ok(parsed_url) => match Request::post(parsed_url) {
                    Err(error) => return Err(MalformedRequest(error, url_str.to_string())),
                    Ok(request) => request
                }
            }
        };
        request.headers_mut().set(ContentLength(body.len()));
        match request.start() {
            Err(error) => return Err(RequestFailed(error, body)),
            Ok(mut request) => match request.write_str(body.as_slice()) {
                Err(error) => return Err(RequestIoFailed(error, body)),
                Ok(_) => match request.send() {
                    Err(error) => return Err(RequestFailed(error, body)),
                    Ok(mut response) => match response.read_to_end() {
                        Err(error) => Err(RequestIoFailed(error, body)),
                        Ok(response_body) => {
                            debug!("Request to {} succeeded", self.url);
                            Ok(response_body)
                        }
                    }
                }
            }
        }

    }

    // extract JSON nodes from response
    fn decode_traversal(source: Vec<u8>, expectation: Expectation) -> GraphResult<Nodes> {
        match str::from_utf8(source.as_slice()) {
            None => Err(ResponseParseFailed),
            Some(traversal_json) => {
                debug!("start decoding \n===\n{}\n===\n", traversal_json);
                match json_decode(traversal_json) {
                    Err(error) => Err(DecodingFailed(error, traversal_json.to_string())),
                    Ok(nodes) => Ok(Nodes(nodes))
                }
            }
        }
    }

}

impl<S: Decoder<E>, E> Decodable<S, E> for Nodes {

    fn decode(decoder: &mut S) -> Result<Nodes, E> {
        decode_nodes(decoder)
    }
}

fn decode_nodes<S: Decoder<E>, E>(decoder: &mut S) -> Result<Nodes, E> {
    decoder.read_struct("__unused__", 0, |decoder| {
        decoder.read_struct_field("result", 0, |decoder| {
            decoder.read_option(|decoder, has_value| {
                match has_value {
                    false => Ok(Nodes(Vec::new())),
                    true => decoder.read_seq(|decoder, len| {
                        let mut nodes: Vec<HashMap<String, String>> = Vec::with_capacity(len);
                        for i in range(0u, len) {
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
    })
}

fn decode_node<S: Decoder<E>, E>(decoder: &mut S) -> Result<HashMap<String, String>, E> {
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
        Ok(data_map)
    })
}
