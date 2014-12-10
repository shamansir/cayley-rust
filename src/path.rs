#![macro_escape]

/// This module is the main entry point to ask for [Nodes](../graph/struct.Nodes.html) from
/// database using [Graph](../graph/struct.Graph.html) as an interceptor. It defines everything
/// what may appear required to construct a query, part of a query, or to re-use some prepared one to find any data
/// in a [Graph](../graph/struct.Graph.html).
///
/// This way, module covers the [Cayley version of Gremlin API](https://github.com/google/cayley/blob/master/docs/GremlinAPI.md),
/// adapted to Rust language.
///
/// ## `vertex!`
///
/// To contruct a one-liner query, use `vertex!` macro:
///
/// ```rust
/// # #![feature(globs)]
/// # #![feature(phase, macro_rules)]
///
/// # #[phase(plugin, link)]
/// # extern crate cayley;
///
/// # use cayley::Graph;
///
/// # use cayley::path::Vertex;
/// # use cayley::path::Traversal::*;
/// # use cayley::path::Final::*;
///
/// # use cayley::selectors::*;
///
/// # fn main() {
/// let graph = Graph::default().unwrap();
/// graph.find(vertex![ Node("foo")
///                     -> As(Tags(vec!("tag-a", "tag-b")))
///                     -> OutP(Predicate("follows"))
///                     => All ]).unwrap();
/// # }
/// ```
///
/// Another example:
///
/// ```rust
/// # #![feature(globs)]
/// # #![feature(phase, macro_rules)]
///
/// # #[phase(plugin, link)]
/// # extern crate cayley;
///
/// # use cayley::{Graph, Nodes};
///
/// # use cayley::path::Vertex;
/// # use cayley::path::Final::*;
///
/// # use cayley::selector::NodeSelector::*;
///
/// # fn main() {
/// let graph = Graph::new("localhost", 64210, DefaultVersion).unwrap();
/// match graph.find(vertex![ AnyNode => All ]) {
///    Ok(Nodes(nodes)) => assert!(nodes.len() > 0),
///    Err(error) => panic!(error.to_string())
/// };
/// # }
/// ```
///
/// Notice that Finals like `All` or `GetLimit(..)` require `=>` symbol to be
/// specified before, while traversals use `->` symbol for chaining. And actually
/// these Finals are the only ones supported in Cayley-HTTP right now.
///
/// The query above looks like this in Cayley/Gremlin syntax: `g.V().As(["tag-a","tag-b"]).Out("follows").All()`
///
/// In general, `vertex!` macro syntax is:
///
/// ```text
/// vertex![ <AnyNode | Node(name) | Nodes([names])>
///          (-> Traversal)*
///          (=> <All | GetLimit(n)>)? ]
/// ```
///
/// JFYI, without a macro usage, this query, expanded in pure Rust code, looks like:
///
/// ```ignore
/// match Vertex::compile_query(AnyNode,
///                             box [ As(Tags(vec!("tag-a", "tag-b"))),
///                                   OutP(Predicate("follows")) ],
///                             All) => {
///     Some(v) => v, None => panic!("Vertex query failed to compile!")
/// }
/// ```
///
/// Vertices may be logically-combined with other routes, but, following to the spec,
/// included route _should not have Final_ at its end:
///
/// ```ignore
/// let v_1 = vertex![ AnyNode -> Out(Predicate("follows"), AnyTag)
///                            -> In(Predicate("follows"), AnyTag) ];
/// let v_2 = vertex![ Node("bar") -> Has(Predicate("status"), Node("cool_person"))
///                                -> And(&v_1) ];
/// graph.find(vertex![ Node("foo") -> Union(&v_2) => All ]).unwrap();
/// ```
///
/// This query will be compiled to:
///
/// `g.V("foo").Or(g.V("bar").Has("status","cool_person").And(g.V().Out("follows").In("follows"))).All()`
///
/// ## `morphism!`
///
/// Morphism helps to store named paths and reuse them later in the same HTTP request
/// (under the hood, next HTTP request will actually "forget" them,
/// but Cayley driver ensures to include used Morhisms in every query/request automatically).
/// In comparison with Vertex-routes, Morphism has no starting node,
/// it is just a named traversal sequence.
///
/// ```ignore
/// let cool_and_follows = morphism![ "c_and_f" -> Has(Predicate("status"), Node("cool_person"))
///                                             -> OutP(Predicate("follows")) ];
/// graph.find(vertex![ Node("foo") -> Follow(&cool_and_follows) => All ]).unwrap();
/// graph.find(vertex![ AnyNode -> FollowR(&cool_and_follows) => GetLimit(10) ]).unwrap();
/// ```
///
/// These queries will be compiled to:
///
/// `var c_and_f=g.M().Has("status","cool_persone").Out("Follows");g.V("foo").Follow(c_and_f).All()`
///
/// and
///
/// `var c_and_f=g.M().Has("status","cool_persone").Out("Follows");g.V().FollowR(c_and_f).GetLimit(10)`
///
/// `morphism!` macro sytax:
///
/// ```text
/// morphism![ "name" (-> Traversal)* ]
/// ```
///
/// ## `path!`
///
/// `path!` macro makes it possible to join two paths (but with no Final):
///
/// ```ignore
/// let cFollows = vertex![ Node("C") -> Out(Predicate("follows"), AnyTag) ];
/// let dFollows = vertex![ Node("D") -> Out(Predicate("follows"), AnyTag) ];
///
/// graph.find(vertex![ AnyNode -> Or(&(cFollows + path![ And(&dFollows) ])) => All ]).unwrap();
/// graph.find(vertex![ AnyNode -> And(&(cFollows + path![ Or(&dFollows) ])) => All ]).unwrap();
/// ```
///
/// `path!` macro syntax:
///
/// ```text
/// path![ (Traversal ->)* Traversal ]
/// ```
///
/// ## Notes
///
/// Since there is no overloading or optional parameters in Rust, some traversals
/// requiring two parameters, were split into three variants to ease the usage:
/// For example, `.Out([predicate],[tag])` was split into three: `.Out(predicate, tag)`,
/// `.OutP(predicate)`, `.OutT(tag)`. Same for `.In` and `.Both`. `.Tag()` was renamed
/// to `TagWith`, to provide a way to call it in a namespace shared with `TagSelector::Tag`.
/// See [Traversal](../path/enum/Traversal.html) for a full list of supported traversals.

use selector::{NodeSelector, TagSelector, PredicateSelector};

use selector::NodeSelector::{AnyNode, Node, Nodes};
use selector::TagSelector::{AnyTag, Tag, Tags};
use selector::PredicateSelector::{AnyPredicate, Predicate, Predicates};
use selector::PredicateSelector::Route as FromRoute;

#[macro_export]
macro_rules! vertex(
    [ $e1:expr $(-> $e2:expr)* => $e3:expr ] => (
        match Vertex::compile_query($e1, box [$($e2,)*], $e3) {
              Some(v) => v, None => panic!("Vertex query failed to compile!")
        }
    );
    [ $e1:expr $(-> $e2:expr)+ ] => (
        match Vertex::compile_route($e1, box [$($e2,)*]) {
            Some(v) => v, None => panic!("Vertex query failed to compile!")
        }
    );
    [ $e1:expr ] => (
        match Vertex::compile_route($e1, box []) {
            Some(v) => v, None => panic!("Vertex query failed to compile!")
        }
    )
)

#[macro_export]
macro_rules! morphism(
    [ $e1:expr $(-> $e2:expr)+ ] => (
        match Morphism::compile_reuse($e1, box [$($e2,)*]) {
            Some(m) => m, None => panic!("Morphism path failed to compile!")
        }
    )
)

#[macro_export]
macro_rules! path(
    [ $e1:expr $(-> $e2:expr)* ] => (
        match Trail::compile_path(box [$e1, $($e2,)*]) {
            Some(m) => m, None => panic!("Trail path failed to compile!")
        }
    )
)

/// Represents a traversal part of a path. Used to contruct both Paths and Queries.
pub enum Traversal<'t> {
    // Basic Trail
    Out(PredicateSelector<'t>, TagSelector<'t>),
    OutP(PredicateSelector<'t>),
    OutT(TagSelector<'t>),
    In(PredicateSelector<'t>, TagSelector<'t>),
    InP(PredicateSelector<'t>),
    InT(TagSelector<'t>),
    Both(PredicateSelector<'t>, TagSelector<'t>),
    BothP(PredicateSelector<'t>),
    BothT(TagSelector<'t>),
    Is(NodeSelector<'t>),
    Has(PredicateSelector<'t>, NodeSelector<'t>),
    // Tagging
    TagWith(TagSelector<'t>),
    As(TagSelector<'t>),
    Back(TagSelector<'t>),
    Save(PredicateSelector<'t>, TagSelector<'t>),
    // Joining
    Intersect(&'t CompiledRoute),
    And(&'t CompiledRoute),
    Union(&'t CompiledRoute),
    Or(&'t CompiledRoute),
    // Morphisms
    Follow(&'t CompiledReuse),
    FollowR(&'t CompiledReuse)
}

/// Represents a final part of a path. Used to contruct Queries.
pub enum Final {
    Undefined,
    All,
    GetLimit(int),
    ToArray,
    ToValue,
    TagArray,
    TagValue
    /* ForEach(|int|:'q -> int) */
    /* Map(|int|:'q -> int) */
}

/// This enum defines which type of a data this Query expects from Graph. Currently,
/// Only `NodeSequence` is supported by Cayley for HTTP requests
pub enum Expectation {
    Unknown,
    SingleNode,
    NodeSequence,
    NameSequence,
    TagSequence,
    SingleTag
}

// ================================ Path, Query & Reuse ===================== //

/// Represents a navigational part of a path, i.e. `.Out("foo").Intersect(bar).Has("buz")`
trait Path: ToString {

    fn compile_path(&self) -> Option<CompiledPath>;

}

/// Represents a non-finalized path together with initial pivot, i.e.
/// `g.V().Out("foo").Intersect(bar).Has("buz")` or
/// `g.M().Out("foo").Intersect(bar).Has("buz")`
trait Route: Path {

    fn compile_route(&self) -> Option<CompiledRoute>;

}

/// Represents a query, i.e. `g.V().Out("foo").Intersect(bar).Has("buz").GetLimit(10)`
trait Query: Route {

    fn compile_query(&self) -> Option<CompiledQuery>;

}

/// Represents a named path, i.e. `out_int_has = g.M().Out("foo").Intersect(bar).Has("buz")`
trait Reuse: Route {

    fn get_name(&self) -> &str;

    fn compile_reuse(&self) -> Option<CompiledReuse>;

}

/// Stores a navigational part of a path, i.e. `.Out("foo").Intersect(bar).Has("buz")`
pub struct CompiledPath {
    pub prefix: String,
    pub value: String
}

/// Stores a non-finalized path together with initial pivot, i.e.
/// `g.V().Out("foo").Intersect(bar).Has("buz")` or
/// `g.M().Out("foo").Intersect(bar).Has("buz")`
pub struct CompiledRoute {
    pub prefix: String,
    pub value: String
}

/// Stores a query, i.e. `g.V().Out("foo").Intersect(bar).Has("buz").GetLimit(10)`
pub struct CompiledQuery {
    pub prefix: String,
    pub value: String,
    pub expectation: Expectation
}

/// Stores a named path, i.e. `out_int_has = g.M().Out("foo").Intersect(bar).Has("buz")`
pub struct CompiledReuse {
    pub prefix: String,
    pub name: String,
    pub value: String
}

impl Add<CompiledPath, CompiledPath> for CompiledPath {

    fn add(&self, _rhs: &CompiledPath) -> CompiledPath {
        CompiledPath { prefix: _rhs.prefix + self.prefix, value: self.value + _rhs.value }
    }

}

impl Add<CompiledPath, CompiledRoute> for CompiledRoute {

    fn add(&self, _rhs: &CompiledPath) -> CompiledRoute {
        CompiledRoute { prefix: _rhs.prefix + self.prefix, value: self.value + _rhs.value }
    }

}

// ================================ Trail ============================= //

/// A structure to hold [Path](../path/trait.Path.html) data before its compilation to
/// [CompiledPath](../path/struct.CompiledPath)
pub struct Trail<'ps>(pub Box<[Traversal<'ps>]>);

impl<'t> Trail<'t> {

    pub fn compile_path<'a>(traversals: Box<[Traversal<'a>]>) -> Option<CompiledPath> {
        Trail(traversals).compile_path()
    }

}

impl<'ts> ToString for Trail<'ts> {

    fn to_string(&self) -> String {
        match self.compile_path() {
            Some(path) => path.value,
            None => "<Trail: Incorrect>".to_string()
        }
    }

}

impl<'p> Path for Trail<'p> {

    fn compile_path(&self) -> Option<CompiledPath> {
        match *self {
            Trail(ref traversals) =>
                Some(CompiledPath {
                    prefix: parse_prefix(traversals),
                    value: parse_traversals(traversals)
                })
        }
    }

}

// ================================ Morphism ================================ //

/// A structure to hold [Reuse](../path/trait.Reuse.html) data before its compilation to
/// [CompiledReuse](../path/struct.CompiledReuse)
pub struct Morphism<'m>(pub &'m str, pub Box<[Traversal<'m>]>);

impl<'m> Morphism<'m> {

    pub fn compile_reuse<'a>(name: &'a str, traversals: Box<[Traversal<'a>]>) -> Option<CompiledReuse> {
        Morphism(name, traversals).compile_reuse()
    }

}

impl<'ts> ToString for Morphism<'ts> {

    fn to_string(&self) -> String {
        match self.compile_reuse() {
            Some(reuse) => {
                reuse.name.to_string() + ":" + reuse.value
            },
            None => "<Morphism: Incorrect>".to_string()
        }
    }

}

impl<'p> Path for Morphism<'p> {

    fn compile_path(&self) -> Option<CompiledPath> {
        match *self {
            Morphism(_, ref traversals) =>
                Some(CompiledPath {
                    prefix: parse_prefix(traversals),
                    value: parse_traversals(traversals)
                })
        }
    }

}

impl<'r> Route for Morphism<'r> {

    fn compile_route(&self) -> Option<CompiledRoute> {
        match *self {
            Morphism(_, ref traversals) =>
                Some(CompiledRoute {
                    prefix: parse_prefix(traversals),
                    value: "g.M()".to_string() + parse_traversals(traversals)
                })
        }
    }

}

impl<'r> Reuse for Morphism<'r> {

    fn get_name(&self) -> &str {
        match *self {
            Morphism(name, _) => name
        }
    }

    fn compile_reuse(&self) -> Option<CompiledReuse> {
        match *self {
            Morphism(name, ref traversals) =>
                Some(CompiledReuse {
                    name: name.to_string(),
                    prefix: parse_prefix(traversals),
                    value: "g.M()".to_string() + parse_traversals(traversals)
                })
        }
    }

}

// ================================ Vertex ================================== //

/// A structure to hold [Query](../path/trait.Query.html) data before its compilation to
/// [CompiledQuery](../path/struct.CompiledQuery) or [CompiledRoute](../path/struct.CompiledRoute)
pub struct Vertex<'v>(pub NodeSelector<'v>, pub Box<[Traversal<'v>]>, pub Final);

impl<'v> Vertex<'v> {

    pub fn compile_query<'a>(start: NodeSelector<'a>, traversals: Box<[Traversal<'a>]>, _final: Final) -> Option<CompiledQuery> {
        Vertex(start, traversals, _final).compile_query()
    }

    pub fn compile_route<'a>(start: NodeSelector<'a>, traversals: Box<[Traversal<'a>]>) -> Option<CompiledRoute> {
        Vertex(start, traversals, Final::Undefined).compile_route()
    }

}

impl<'ts> ToString for Vertex<'ts> {

    fn to_string(&self) -> String {
        match self.compile_query() {
            Some(query) => query.value,
            None => "<Vertex: Incorrect>".to_string()
        }

    }

}

impl<'p> Path for Vertex<'p> {

    fn compile_path(&self) -> Option<CompiledPath> {
        match *self {
            Vertex(_, ref traversals, _) =>
                Some(CompiledPath {
                    prefix: parse_prefix(traversals),
                    value: parse_traversals(traversals)
                })
        }
    }

}

impl<'r> Route for Vertex<'r> {

    fn compile_route(&self) -> Option<CompiledRoute> {
        match *self {
            Vertex(ref start, ref traversals, _) =>
                Some(CompiledRoute {
                    prefix: parse_prefix(traversals),
                    value: parse_start(start) + parse_traversals(traversals)
                })
        }
    }

}

impl<'q> Query for Vertex<'q> {

    fn compile_query(&self) -> Option<CompiledQuery> {
        match *self {
            Vertex(ref start, ref traversals, _final) => {
                let prefix = parse_prefix(traversals);
                let mut value = String::new();
                value.push_str(parse_start(start).as_slice());
                value.push_str(parse_traversals(traversals).as_slice());
                match _final {
                    Final::Undefined => {},
                    _ => value.push_str(parse_final(_final).as_slice())
                }
                Some(CompiledQuery {
                    prefix: prefix,
                    value: value,
                    expectation: match _final {
                        Final::Undefined    => Expectation::Unknown,
                        Final::All          => Expectation::NodeSequence,
                        Final::GetLimit(..) => Expectation::NodeSequence,
                        Final::ToArray      => Expectation::NameSequence,
                        Final::ToValue      => Expectation::SingleNode,
                        Final::TagArray     => Expectation::TagSequence,
                        Final::TagValue     => Expectation::SingleTag
                    }
                })
            }
        }
    }

}

// ================================ parsing ================================= //

fn parse_prefix(traversals: &Box<[Traversal]>) -> String {
    let mut result = String::new();
    for traversal in traversals.iter() {
        match *traversal {
            Traversal::Follow(reusable) | Traversal::FollowR(reusable) => {
                result.push_str(reusable.prefix.as_slice());
                result.push_str(format!("var {name} = {path};",
                                        name = reusable.name,
                                        path = reusable.value).as_slice());
            },
            Traversal::Intersect(query) | Traversal::And(query) | Traversal::Union(query) | Traversal::Or(query) => {
                result.push_str(query.prefix.as_slice());
            },
            _ => {}
        }
    }
    result
}

fn parse_start(start: &NodeSelector) -> String {
    match *start {
        AnyNode => "g.V()".to_string(),
        Node(name) => format!("g.V(\"{0}\")", name),
        Nodes(ref names) => format!("g.V(\"{0}\")", names.connect("\",\""))
    }
}

fn parse_traversals(traversals: &Box<[Traversal]>) -> String {
    let mut result = String::new();
    for traversal in traversals.iter() {
        result.push_str(match *traversal {
            /* FIXME: Traversal:: shouldn't be required */
            // Basic Trail ================================================================================================
            Traversal::Out(ref predicates, ref tags)   => format!(".Out({})",  parse_predicates_and_tags(predicates, tags)),
            Traversal::OutP(ref predicates)            => format!(".Out({})",  parse_predicates_and_tags(predicates, &AnyTag)),
            Traversal::OutT(ref tags)                  => format!(".Out({})",  parse_predicates_and_tags(&AnyPredicate, tags)),
            Traversal::In(ref predicates, ref tags)    => format!(".In({})",   parse_predicates_and_tags(predicates, tags)),
            Traversal::InP(ref predicates)             => format!(".In({})",   parse_predicates_and_tags(predicates, &AnyTag)),
            Traversal::InT(ref tags)                   => format!(".In({})",   parse_predicates_and_tags(&AnyPredicate, tags)),
            Traversal::Both(ref predicates, ref tags)  => format!(".Both({})", parse_predicates_and_tags(predicates, tags)),
            Traversal::BothP(ref predicates)           => format!(".Both({})", parse_predicates_and_tags(predicates, &AnyTag)),
            Traversal::BothT(ref tags)                 => format!(".Both({})", parse_predicates_and_tags(&AnyPredicate, tags)),
            Traversal::Is(ref nodes)                   => match nodes {
                                                              &AnyNode => ".Is()".to_string(),
                                                              &Node(name) => format!(".Is(\"{}\")", name),
                                                              &Nodes(ref names) => format!(".Is(\"{}\")", names.connect("\",\""))
                                                          },
            Traversal::Has(ref predicates, ref nodes)  => format!(".Has({})", parse_predicates_and_nodes(predicates, nodes)),
            // Tagging =========================================================================================================
            Traversal::TagWith(ref tags) |
            Traversal::As(ref tags)                    => match tags {
                                                              &AnyTag => ".As()".to_string(),
                                                              &Tag(name) => format!(".As(\"{}\")", name),
                                                              &Tags(ref names) => format!(".As(\"{}\")", names.connect("\",\""))
                                                          },
            Traversal::Back(ref tags)                  => match tags {
                                                              &AnyTag => ".Back()".to_string(),
                                                              &Tag(name) => format!(".Back(\"{}\")", name),
                                                              &Tags(ref names) => format!(".Back(\"{}\")", names.connect("\",\""))
                                                          },
            Traversal::Save(ref predicates, ref tags)  => format!(".Save({})", parse_predicates_and_tags(predicates, tags)),
            // Joining =========================================================================================================
            Traversal::Intersect(query) |
            Traversal::And(query)                      => format!(".And({})", query.value),
            Traversal::Union(query) |
            Traversal::Or(query)                       => format!(".Or({})", query.value),
            // Morphisms =======================================================================================================
            Traversal::Follow(reusable)                => format!(".Follow({})", reusable.name),
            Traversal::FollowR(reusable)               => format!(".FollowR({})", reusable.name)
        }.as_slice());
    }
    result
}

fn parse_final(_final: Final) -> String {
    match _final {
        /* FIXME: Final:: shouldn't be required */
        Final::Undefined => "".to_string(),
        Final::All => ".All()".to_string(),
        Final::GetLimit(n) => format!(".GetLimit({})", n),
        Final::ToArray => ".ToArray()".to_string(),
        Final::ToValue => ".ToValue()".to_string(),
        Final::TagArray => ".TagArray()".to_string(),
        Final::TagValue => ".TagValue()".to_string()
    }
}

fn parse_predicates_and_tags(predicates: &PredicateSelector, tags: &TagSelector) -> String {
    match (predicates, tags) {

        (&AnyPredicate, &AnyTag) => "".to_string(),
        (&AnyPredicate, &Tag(tag)) => format!("null,\"{0}\"", tag),
        (&AnyPredicate, &Tags(ref tags)) => format!("null,[\"{0}\"]", tags.connect("\",\"")),

        (&Predicate(predicate), &AnyTag) => format!("\"{0}\"", predicate),
        (&Predicate(predicate), &Tag(tag)) =>
            format!("\"{0}\",\"{1}\"", predicate, tag),
        (&Predicate(predicate), &Tags(ref tags)) =>
            format!("\"{0}\",[\"{1}\"]", predicate, tags.connect("\",\"")),

        (&Predicates(ref predicates), &AnyTag) =>
            format!("[\"{0}\"]", predicates.connect("\",\"")),
        (&Predicates(ref predicates), &Tag(tag)) =>
            format!("[\"{0}\"],\"{1}\"", predicates.connect("\",\""), tag),
        (&Predicates(ref predicates), &Tags(ref tags)) =>
            format!("[\"{0}\"],[\"{1}\"]", predicates.connect("\",\""), tags.connect("\",\"")),

        (&FromRoute(route), &AnyTag) => route.value.clone(),
        (&FromRoute(route), &Tag(tag)) =>
            format!("{0}, \"{1}\"", route.value, tag),
        (&FromRoute(route), &Tags(ref tags)) =>
            format!("{0}, [\"{1}\"]", route.value, tags.connect("\",\""))

    }
}

fn parse_predicates_and_nodes(predicates: &PredicateSelector, nodes: &NodeSelector) -> String {
    match (predicates, nodes) {

        (&AnyPredicate, &AnyNode) => "".to_string(),
        (&AnyPredicate, &Node(node)) => format!("null,\"{0}\"", node),
        (&AnyPredicate, &Nodes(ref nodes)) => format!("null,[\"{0}\"]", nodes.connect("\",\"")),

        (&Predicate(predicate), &AnyNode) => format!("\"{0}\"", predicate),
        (&Predicate(predicate), &Node(tag)) =>
            format!("\"{0}\",\"{1}\"", predicate, tag),
        (&Predicate(predicate), &Nodes(ref nodes)) =>
            format!("\"{0}\",[\"{1}\"]", predicate, nodes.connect("\",\"")),

        (&Predicates(ref predicates), &AnyNode) =>
            format!("[\"{0}\"]", predicates.connect("\",\"")),
        (&Predicates(ref predicates), &Node(node)) =>
            format!("[\"{0}\"],\"{1}\"", predicates.connect("\",\""), node),
        (&Predicates(ref predicates), &Nodes(ref nodes)) =>
            format!("[\"{0}\"],[\"{1}\"]", predicates.connect("\",\""), nodes.connect("\",\"")),

        (&FromRoute(route), &AnyNode) => route.value.clone(),
        (&FromRoute(route), &Node(node)) =>
            format!("{0},\"{1}\"", route.value, node),
        (&FromRoute(route), &Nodes(ref nodes)) =>
            format!("{0},[\"{1}\"]", route.value, nodes.connect("\",\""))

    }
}
