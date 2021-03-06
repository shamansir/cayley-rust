#![crate_name = "cayley"]

#![doc(html_root_url = "http://shamansir.github.io/cayley-rust")]

//! <style>ul a { color: #4e8bca; }</style>
//!
//! # Google Cayley Database Driver
//!
//! [Hosted at Github](https://github.com/shamansir/cayley-rust).
//!
//! Jump to: [Graph](./graph/struct.Graph.html) |
//!          [Vertex](./path/struct.Vertex.html) |
//!          [Morphism](./path/struct.Morphism.html) |
//!          [Traversal](./graph/enum.Traversal.html)
//!
//! ## Connection
//!
//! To connect to a graph, start Cayley itself:
//!
//! `$ ./cayley http --dbpath=<your-database>`
//!
//! Then, bind driver to your host this way:
//!
//! ```ignore
//! use cayley::{Graph, DefaultVersion};
//! let graph = match Graph::new("localhost", 64210, DefaultVersion) {
//!    Err(error) => panic!(error),
//!    Ok(graph) => graph
//! };
//! ```
//!
//! For the moment, this code performs no connection at all, when you only create a Graph.
//! On the other hand, the connection is established for every query. So this error, if happened,
//! is not telling that connection was failed here, it just tells about malformed URL.
//! But things may change, and even when they'll do, you still have a chance to
//! pattern-match the error, if you need.
//!
//! ## Query
//!
//! Query pattern looks like this:
//!
//! ```ignore
//! use cayley::{Graph, DefaultVersion};
//! use cayley::GraphNodes;
//! use cayley::path::{Vertex, Query}; // Query trait import is required
//! use cayley::selector::AnyNode;
//!
//! let graph = Graph::new("localhost", 64210, DefaultVersion).unwrap();
//! match graph.find(Vertex::start(AnyNode).All()) {
//!    Ok(GraphNodes(nodes)) => assert!(nodes.len() > 0),
//!    Err(error) => panic!(error.to_string()),
//! };
//! ```
//!
//! So in general it looks like `graph.find(<Query>)`.
//!
//! [GraphNodes](./graph/struct.GraphNodes.html) is a wrapper for `Vec<GraphNode>`.
//! [GraphNode](./graph/struct.GraphNode.html) is a wrapper for `HashMap<String, String>`
//!
//! Morphism used this way:
//!
//! ```ignore
//! #![allow(unused_result)]
//! use cayley::{Graph, DefaultVersion};
//! use cayley::path::Vertex as V;
//! use cayley::path::Morphism as M;
//! use cayley::path::{Path, Query}; // both traits imports are required
//! use cayley::selector::{Predicate, Node};
//!
//! let graph = Graph::new("localhost", 64210, DefaultVersion).unwrap();
//! let mut follows_m = M::start("foo");
//!         follows_m.OutP(Predicate("follows"));
//! graph.find(V::start(Node("C"))
//!              .Follow(&follows_m)
//!              .Has(Predicate("status"), Node("cool_person"))
//!              .All()).unwrap();
//! ```
//!
//! ## API
//!
//! [Gremlin API](https://github.com/google/cayley/blob/master/docs/GremlinAPI.md) is implemented
//! through these entry points:
//!
//! * [Graph](./graph/struct.Graph.html) provides `.find(<Query>)`;
//! * [Vertex](./path/struct.Vertex.html) provides:
//!     * [Path](./path/trait.Path.html) implemetation with `.Out(...)`, `.In(...)`, `.Has(...)`, `.Or(...)`, `.Follow(...)`, ...
//!     * [Query](./path/trait.Query.html) implemetation with `.All()`, `.GetLimit(...)`, ...
//! * [Morphism](./path/struct.Morphism.html) provides:
//!     * [Path](./path/trait.Path.html) implemetation with `.Out(...)`, `.In(...)`, `.Has(...)`, `.Or(...)`, `.Follow(...)`, ...
//!
//! Follow the links above for a complete lists of methods and to get more information
//! about every mentioned structure.

#[doc(no_inline)]
extern crate hyper;

#[doc(no_inline)]
extern crate url;

#[doc(no_inline)]
extern crate rustc_serialize;

#[macro_use]
extern crate log;

pub mod graph;

pub use graph::{Graph, Nodes};
pub use graph::APIVersion::{V1, DefaultVersion};

mod selector;

pub mod error;
pub mod path;

pub use error::Result;
pub use error::Error;

pub mod selectors {
    pub use selector::NodeSelector::*;
    pub use selector::PredicateSelector::*;
    pub use selector::TagSelector::*;
}

pub mod paths {
    pub use path::{Trail, Vertex, Morphism};
    pub use path::Traversal::*;
    pub use path::Final::*;
}
