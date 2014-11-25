#![macro_escape]

use std::fmt::{Show, Formatter, FormatError};

use selector::{NodeSelector, TagSelector, PredicateSelector};

use selector::{AnyNode, Node, Nodes};
use selector::{AnyTag, Tag, Tags};
use selector::{AnyPredicate, Predicate, Predicates};
use selector::Query as FromQuery;

#[macro_export]
macro_rules! vertex(
    [ $e1:expr $(-> $e2:expr)* => $e3:expr ] => (
        &Vertex($e1, box [$($e2,)*], $e3)
    )
)
/* macro_rules! enum_macro(
    [ $($e1:ident)->+ => $e2:ident ] => (
        vec!($($e1,)+), $e2
    )
) */

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
    // Tag(TagSelector<'t>): TagSelector has the same name
    As(TagSelector<'t>),
    Back(TagSelector<'t>),
    Save(PredicateSelector<'t>, TagSelector<'t>),
    // Joining
    Intersect(&'t Query+'t),
    And(&'t Query+'t),
    Union(&'t Query+'t),
    Or(&'t Query+'t),
    // Morphisms
    Follow(&'t Path+'t),
    FollowR(&'t Path+'t)
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
    ExpectationUnknown,
    ExpectSingleNode,
    ExpectNodeSequence,
    ExpectNameSequence,
    ExpectTagSequence,
    ExpectSingleTag
}

// ================================ Path & Query ============================ //

pub trait Path: ToString { }

pub trait Query: Path {

    fn compile(&self) -> Option<(String, Expectation)>;

}

// ================================ Morphism ================================ //

pub struct Morphism<'m>(pub &'m str, pub Box<[Traversal<'m>]>);

impl<'ts> ToString for Morphism<'ts> {

    fn to_string(&self) -> String { String::new() }

}

impl<'p> Path for Morphism<'p> { }

// ================================ Vertex ================================== //

pub struct Vertex<'v>(pub NodeSelector<'v>, pub Box<[Traversal<'v>]>, pub Final);

impl<'ts> ToString for Vertex<'ts> {

    fn to_string(&self) -> String {
        match *self {
            Vertex(ref start, traversals, _final) => {
                let mut result = String::with_capacity(15);
                result.push_str(match *start {
                    AnyNode => "g.V()".to_string(), // FIXME: double-conversion here?
                    Node(name) => format!("g.V(\"{0}\")", name),
                    Nodes(ref names) => format!("g.V(\"{0}\")", names.connect("\",\""))
                }.as_slice());
                result
            }
        }
    }

}

impl<'p> Path for Vertex<'p> { }

impl<'q> Query for Vertex<'q> {

    fn compile(&self) -> Option<(String, Expectation)> {
        Some((self.to_string(), match *self {
            Vertex(_, _, _final) => {
                match _final {
                    All => ExpectNodeSequence,
                    GetLimit(_) => ExpectNodeSequence,
                    ToArray => ExpectNameSequence,
                    ToValue => ExpectSingleNode,
                    TagArray => ExpectTagSequence,
                    TagValue => ExpectSingleTag
                }
            }
        }))
    }

}

// ================================ utils =================================== //

fn predicates_and_tags(predicates: PredicateSelector, tags: TagSelector) -> String {
    match (predicates, tags) {

        (AnyPredicate, AnyTag) => "".to_string(),
        (AnyPredicate, Tag(tag)) => format!("null,\"{0}\"", tag),
        (AnyPredicate, Tags(tags)) => format!("null,[\"{0}\"]", tags.connect("\",\"")),

        (Predicate(predicate), AnyTag) => format!("\"{0}\"", predicate),
        (Predicate(predicate), Tag(tag)) =>
            format!("\"{0}\",\"{1}\"", predicate, tag),
        (Predicate(predicate), Tags(tags)) =>
            format!("\"{0}\",[\"{1}\"]", predicate, tags.connect("\",\"")),

        (Predicates(predicates), AnyTag) =>
            format!("[\"{0}\"]", predicates.connect("\",\"")),
        (Predicates(predicates), Tag(tag)) =>
            format!("[\"{0}\"],\"{1}\"", predicates.connect("\",\""), tag),
        (Predicates(predicates), Tags(tags)) =>
            format!("[\"{0}\"],[\"{1}\"]", predicates.connect("\",\""), tags.connect("\",\"")),

        (FromQuery(query), AnyTag) =>
            match query.compile() {
                Some((compiled, _)) => compiled,
                None => "null".to_string()
            },
        (FromQuery(query), Tag(tag)) =>
            format!("{0}, \"{1}\"",
                    match query.compile() {
                        Some((compiled, _)) => compiled,
                        None => "null".to_string()
                    },
                    tag),
        (FromQuery(query), Tags(tags)) =>
            format!("{0}, [\"{1}\"]",
                    match query.compile() {
                        Some((compiled, _)) => compiled,
                        None => "null".to_string()
                    },
                    tags.connect("\",\""))

    }
}

fn predicates_and_nodes(predicates: PredicateSelector, nodes: NodeSelector) -> String {
    match (predicates, nodes) {

        (AnyPredicate, AnyNode) => "".to_string(),
        (AnyPredicate, Node(node)) => format!("null,\"{0}\"", node),
        (AnyPredicate, Nodes(nodes)) => format!("null,[\"{0}\"]", nodes.connect("\",\"")),

        (Predicate(predicate), AnyNode) => format!("\"{0}\"", predicate),
        (Predicate(predicate), Node(tag)) =>
            format!("\"{0}\",\"{1}\"", predicate, tag),
        (Predicate(predicate), Nodes(nodes)) =>
            format!("\"{0}\",[\"{1}\"]", predicate, nodes.connect("\",\"")),

        (Predicates(predicates), AnyNode) =>
            format!("[\"{0}\"]", predicates.connect("\",\"")),
        (Predicates(predicates), Node(node)) =>
            format!("[\"{0}\"],\"{1}\"", predicates.connect("\",\""), node),
        (Predicates(predicates), Nodes(nodes)) =>
            format!("[\"{0}\"],[\"{1}\"]", predicates.connect("\",\""), nodes.connect("\",\"")),

        (FromQuery(query), AnyNode) =>
            match query.compile() {
                Some((compiled, _)) => compiled,
                None => "null".to_string()
            },
        (FromQuery(query), Node(node)) =>
            format!("{0},\"{1}\"",
                    match query.compile() {
                        Some((compiled, _)) => compiled,
                        None => "null".to_string()
                    },
                    node),
        (FromQuery(query), Nodes(nodes)) =>
            format!("{0},[\"{1}\"]",
                    match query.compile() {
                        Some((compiled, _)) => compiled,
                        None => "null".to_string()
                    },
                    nodes.connect("\",\""))

    }
}
