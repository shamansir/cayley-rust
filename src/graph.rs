use std::str;

use url::Url;
use http::client::RequestWriter;
use http::method::Post;

use serialize::{Decoder, Decodable};
use serialize::json::decode as json_decode;

use std::collections::HashMap;

use path::Query;
use path::Reuse;

use errors::{GraphResult,
             InvalidUrl, MalformedRequest, RequestFailed,
             DecodingFailed, ResponseParseFailed,
             QueryNotFinalized, QueryCompilationFailed,
             ReusableCannotBeSaved };

/// Provides access to currently running Cayley database, among with
/// an ability to run queries there, and to write there your data
/// (honestly, only if there's a `graph.emit()` method below—if not,
/// it will just soon be there).
///
/// * Use `Graph::default()` to connect to `localhost:64210`.
/// * Use `Graph::new(host, port, api_version)` to specify the location of database manually.
///
/// * Use `Graph::find(<Query>)` to find anything using [Query](./path/trait.Query.html) trait implementor
/// (`Query`, for example, is implemented by [Vertex](./path/struct.Vertex.html)), which in its turn
/// is similar to [Gremlin API](https://github.com/google/cayley/blob/master/docs/GremlinAPI.md).
/// * Use `Graph::find_by(<String>)` to find anything using [Gremlin API](https://github.com/google/cayley/blob/master/docs/GremlinAPI.md) API
/// from a prepared string. A raw, but not so beautiful, way to execute query.
/// * Use `Graph::save(<Path>)` to save a [Morphism](./path/struct.Morphism.html).
pub struct Graph {
    url: String
}

/// A wrapper for a single item Cayley returned in response for a query
pub struct GraphNode(pub HashMap<String, String>);

/// A collection of GraphNode instances
pub struct GraphNodes(pub Vec<GraphNode>);

/// Cayley API Version, planned to default to the latest, if it will ever change
pub enum CayleyAPIVersion { V1, DefaultVersion }

impl Graph {

    /// Create a Graph which connects to the latest API at `localhost:64210`
    pub fn default() -> GraphResult<Graph> {
        Graph::new("localhost", 64210, DefaultVersion)
    }

    /// Create a Graph which connects to the host you specified manually
    pub fn new(host: &str, port: int, version: CayleyAPIVersion) -> GraphResult<Graph> {
        let version_str = match version { V1 | DefaultVersion => "v1" };
        let url = format!("http://{:s}:{:d}/api/{:s}/query/gremlin",
                          host, port, version_str);
        Ok(Graph{ url: url })
    }

    /// Find nodes with the Query implementation (say, Vertex-path) and return them parsed
    ///
    /// Since only [Vertex](./path/struct.Vertex.html) implements [Query](./path/trait.Query.html) trait
    /// following current spec, your code will look like that:
    ///
    /// ```
    /// graph.find(Vertex::start(Node("foo")).InP(Predicate("bar")).All()));
    /// ```
    pub fn find(&self, query: &Query) -> GraphResult<GraphNodes> {
        if query.is_finalized() {
            match query.compile() {
                Some(compiled) => self.find_by(compiled),
                None => Err(QueryCompilationFailed)
            }
        } else { Err(QueryNotFinalized) }
    }

    /// Find nodes using raw pre-compiled query string and return them parsed
    ///
    /// If you want to run just the pure stringified Gremlin queries, bypassing
    /// the string concatenation performed with `path::` module members, this
    /// method is for you.
    ///
    /// ```
    /// graph.exec("g.V(\"foo\").In(\"bar\").All()");
    /// ```
    pub fn exec(&self, query: String) -> GraphResult<GraphNodes> {
        match self.perform_request(query) {
            Ok(body) => Graph::decode_nodes(body),
            Err(error) => Err(error)
        }
    }

    /// Save Morphism or any [Reuse](./path/trait.Reuse.html) implementor in the
    /// database, equivalent to Gremin's `var foo = g.Morphism()...`
    ///
    /// The difference is in the fact you set the name when you create a `Morphism` instance
    /// and then just pass it here, like:
    ///
    /// ```
    /// let m = Morphism::start("foo");
    /// m.Out(Predicate("follows"), AnyTag);
    /// graph.save(m);
    /// ```
    ///
    /// Currently no check is performed if Morphism was already saved or not to
    /// this graph, though it provides `is_saved()` method which may tell if this Morphism
    /// instance was saved at least once to _some_ graph. If in future this check will be
    /// performed, this method definiton won't change, only there will be a new error type.
    pub fn save(&self, reusable: &mut Reuse) -> GraphResult<()> {
        match reusable.save() {
            Some(query) => {
                match self.perform_request(query) {
                    /* TODO: saved flag should know a graph where this morphism was saved */
                    Ok(_) => { reusable.set_saved(); Ok(()) },
                    Err(error) => Err(error)
                }
            },
            None => Err(ReusableCannotBeSaved)
        }
    }

    /// Save Morphism or any [Reuse](./path/trait.Reuse.html) implementor in the
    /// database, under the different name than the one used when it was created
    ///
    /// ```
    /// let m = Morphism::start("foo");
    /// m.Out(Predicate("follows"), AnyTag);
    /// graph.save_as(m, "bar");
    /// ```
    ///
    /// Currently no check is performed if Morphism was already saved or not to
    /// this graph, though it provides `is_saved()` method which may tell if this Morphism
    /// instance was saved at least once to _some_ graph. If in future this check will be
    /// performed, this method definiton won't change, only there will be a new error type.
    pub fn save_as(&self, name: &str, reusable: &mut Reuse) -> GraphResult<()> {
        match reusable.save_as(name) {
            Some(query) => {
                match self.perform_request(query) {
                    /* TODO: saved flag should know a graph where this morphism was saved */
                    Ok(_) => { reusable.set_saved(); Ok(()) },
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
                decoder.read_option(|decoder, has_value| {
                    match has_value {
                        false => Ok(GraphNodes(Vec::new())), /* FIXME: return GraphNodes(None) */
                        true => decoder.read_seq(|decoder, len| {
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
                    }
                })
            })
        })
    }
}
