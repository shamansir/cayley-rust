#![macro_escape]

use std::fmt::{Show, Formatter};
use std::fmt::Error as FormatError;

use selector::{NodeSelector, TagSelector, PredicateSelector};

use selector::NodeSelector::{AnyNode, Node, Nodes};
use selector::TagSelector::{AnyTag, Tag, Tags};
use selector::PredicateSelector::{AnyPredicate, Predicate, Predicates};
use selector::PredicateSelector::Query as FromQuery;

#[macro_export]
macro_rules! vertex(
    [ $e1:expr $(-> $e2:expr)* => $e3:expr ] => (
        &Vertex($e1, box [$($e2,)*], $e3)
    )
)

#[macro_export]
macro_rules! morphism(
    [ $e1:expr $(-> $e2:expr)* ] => (
        &Morphism($e1, box [$($e2,)*])
    )
)

pub enum Traversal<'t> {
    // Basic Traversals
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
    Tag(TagSelector<'t>),
    As(TagSelector<'t>),
    Back(TagSelector<'t>),
    Save(PredicateSelector<'t>, TagSelector<'t>),
    // Joining
    Intersect(&'t Query+'t),
    And(&'t Query+'t),
    Union(&'t Query+'t),
    Or(&'t Query+'t),
    // Morphisms
    Follow(&'t Reuse+'t),
    FollowR(&'t Reuse+'t)
}

pub enum Final {
    All,
    GetLimit(int),
    ToArray,
    ToValue,
    TagArray,
    TagValue
    /* ForEach(|int|:'q -> int) */
    /* Map(|int|:'q -> int) */
}

pub enum Expectation {
    Unknown,
    SingleNode,
    NodeSequence,
    NameSequence,
    TagSequence,
    SingleTag
}

// ================================ Path & Query ============================ //

trait Path: ToString { }

trait Query: Path {

    fn compile(&self) -> Option<(String, Expectation)>;

}

trait Reuse: Path {

    fn get_name(&self) -> &str;

    fn compile(&self) -> Option<(&str, String)>;

}

pub struct CompiledPath {
    path: String
}

pub struct CompiledQuery {
    query: String,
    expectation: Expectation
}

pub struct CompiledReuse {
    name: String,
    path: String
}

// ================================ Morphism ================================ //

pub struct Morphism<'m>(pub &'m str, pub Box<[Traversal<'m>]>);

impl<'m> Morphism<'m> {

    fn compile<'a>(name: &'a str, traversals: Box<[Traversal<'a>]>) -> Option<CompiledReuse> {
        match Morphism(name, traversals).compile() {
            Some((name, path)) => Some(CompiledReuse { name: name.to_string(), path: path }),
            None => None
        }
    }

}

impl<'ts> ToString for Morphism<'ts> {

    fn to_string(&self) -> String {
        match self.compile() {
            Some((name, path)) => {
                name.to_string() + ":" + path
            },
            None => "<Morphism: Incorrect>".to_string()
        }
    }

}

impl<'p> Path for Morphism<'p> { }

impl<'r> Reuse for Morphism<'r> {

    fn get_name(&self) -> &str {
        match *self {
            Morphism(name, _) => name
        }
    }

    fn compile(&self) -> Option<(&str, String)> {
        match *self {
            Morphism(name, ref traversals) => Some((name, parse_traversals(traversals)))
        }
    }

}

// ================================ Vertex ================================== //

pub struct Vertex<'v>(pub NodeSelector<'v>, pub Box<[Traversal<'v>]>, pub Final);

impl<'v> Vertex<'v> {

    fn compile<'a>(start: NodeSelector<'a>, traversals: Box<[Traversal<'a>]>, _final: Final) -> Option<CompiledQuery> {
        match Vertex(start, traversals, _final).compile() {
            Some((query, expectation)) => Some(CompiledQuery { query: query, expectation: expectation }),
            None => None
        }
    }

}

impl<'ts> ToString for Vertex<'ts> {

    fn to_string(&self) -> String {
        match self.compile() {
            Some((query, _)) => query,
            None => "<Vertex: Incorrect>".to_string()
        }

    }

}

impl<'p> Path for Vertex<'p> { }

impl<'q> Query for Vertex<'q> {

    fn compile(&self) -> Option<(String, Expectation)> {
        match *self {
            Vertex(ref start, ref traversals, _final) => {
                let mut result = String::new();
                for traversal in traversals.iter() {
                    match *traversal {
                        Traversal::Follow(reuse) | Traversal::FollowR(reuse) => {
                            match reuse.compile() {
                                Some((name, path)) => {
                                    result.push_str("var ");
                                    result.push_str(name);
                                    result.push_str("=");
                                    result.push_str(path.as_slice());
                                    result.push_str(";");
                                },
                                None => panic!("Reusable passed to .Follow or .FollowR failed to compile")
                            }
                        }
                        _ => {}
                    }
                }
                result.push_str(parse_start(start).as_slice());
                result.push_str(parse_traversals(traversals).as_slice());
                result.push_str(parse_final(_final).as_slice());
                Some((result, match _final {
                    Final::All          => Expectation::NodeSequence,
                    Final::GetLimit(..) => Expectation::NodeSequence,
                    Final::ToArray      => Expectation::NameSequence,
                    Final::ToValue      => Expectation::SingleNode,
                    Final::TagArray     => Expectation::TagSequence,
                    Final::TagValue     => Expectation::SingleTag
                }))
            }
        }
    }

}

// ================================ parsing ================================= //

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
            // Basic Traversals ================================================================================================
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
                                                              &Nodes(ref names) => format!(".Is(\"{}\")", names.connect(","))
                                                          },
            Traversal::Has(ref predicates, ref nodes)  => format!(".Has({})", parse_predicates_and_nodes(predicates, nodes)),
            // Tagging =========================================================================================================
            Traversal::Tag(ref tags) |
            Traversal::As(ref tags)                    => match tags {
                                                              &AnyTag => ".Tag()".to_string(),
                                                              &Tag(name) => format!(".Tag(\"{}\")", name),
                                                              &Tags(ref names) => format!(".Tag(\"{}\")", names.connect(","))
                                                          },
            Traversal::Back(ref tags)                  => match tags {
                                                              &AnyTag => ".Back()".to_string(),
                                                              &Tag(name) => format!(".Back(\"{}\")", name),
                                                              &Tags(ref names) => format!(".Back(\"{}\")", names.connect(","))
                                                          },
            Traversal::Save(ref predicates, ref tags)  => format!(".Save({})", parse_predicates_and_tags(predicates, tags)),
            // Joining =========================================================================================================
            Traversal::Intersect(query) |
            Traversal::And(query)                      => match query.compile() {
                                                              Some((query_str, _)) => format!(".And({})", query.to_string()),
                                                              None => panic!("Traversal passed to .Intersect or .And failed to compile")
                                                          },
            Traversal::Union(query) |
            Traversal::Or(query)                       => match query.compile() {
                                                              Some((query_str, _)) => format!(".And({})", query.to_string()),
                                                              None => panic!("Traversal passed to .Union or .Or failed to compile")
                                                          },
            // Morphisms =======================================================================================================
            Traversal::Follow(reusable)                => format!(".Follow({})", reusable.get_name()),
            Traversal::FollowR(reusable)               => format!(".FollowR({})", reusable.get_name())
        }.as_slice());
    }
    result
}

fn parse_final(_final: Final) -> String {
    match _final {
        /* FIXME: Final:: shouldn't be required */
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

        (&FromQuery(query), &AnyTag) =>
            match query.compile() {
                Some((compiled, _)) => compiled,
                None => "null".to_string()
            },
        (&FromQuery(query), &Tag(tag)) =>
            format!("{0}, \"{1}\"",
                    match query.compile() {
                        Some((compiled, _)) => compiled,
                        None => "null".to_string()
                    },
                    tag),
        (&FromQuery(query), &Tags(ref tags)) =>
            format!("{0}, [\"{1}\"]",
                    match query.compile() {
                        Some((compiled, _)) => compiled,
                        None => "null".to_string()
                    },
                    tags.connect("\",\""))

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

        (&FromQuery(query), &AnyNode) =>
            match query.compile() {
                Some((compiled, _)) => compiled,
                None => "null".to_string()
            },
        (&FromQuery(query), &Node(node)) =>
            format!("{0},\"{1}\"",
                    match query.compile() {
                        Some((compiled, _)) => compiled,
                        None => "null".to_string()
                    },
                    node),
        (&FromQuery(query), &Nodes(ref nodes)) =>
            format!("{0},[\"{1}\"]",
                    match query.compile() {
                        Some((compiled, _)) => compiled,
                        None => "null".to_string()
                    },
                    nodes.connect("\",\""))

    }
}
