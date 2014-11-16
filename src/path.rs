use std::fmt::{Show, Formatter, FormatError};

use selector::{NodeSelector, TagSelector, PredicateSelector};

use selector::{AnyNode, Node, Nodes};
use selector::{AnyTag, Tag, Tags};
use selector::{AnyPredicate, Predicate, Predicates};
use selector::Query as FromQuery;

#[macro_export]
macro_rules! vertex(
    [ $e1:ident $(-> $e2:ident)* => $e3:ident ] => (
        Vertex($e1, &[$($e2,)*], $e3)
    )
)
/* macro_rules! enum_macro(
    [ $($e1:ident)->+ => $e2:ident ] => (
        vec!($($e1,)+), $e2
    )
) */

#[macro_export]
macro_rules! morphism(
    [ $e1:ident $(-> $e2:ident)* ] => (
        Morphism($e1, &[$($e2,)*])
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

pub struct Morphism<'m>(&'m[Traversal<'m>]);

impl<'ts> ToString for Morphism<'ts> {

    fn to_string(&self) -> String { String::new() }

}

impl<'p> Path for Morphism<'p> { }

// ================================ Vertex ================================== //

pub struct Vertex<'v>(NodeSelector<'v>, &'v[Traversal<'v>], Final);

impl<'ts> ToString for Vertex<'ts> {

    fn to_string(&self) -> String {
        match *self {
            Vertex(ref start, traversals, _final) => {
                let mut result = String::with_capacity(15);
                result.push_str(match *start {
                    AnyNode => "g.V()".to_string(), // FIXME: double-conversion here?
                    Node(name) => format!("g.V(\"{:s}\")", name),
                    Nodes(ref names) => format!("g.V(\"{:s}\")", names.connect("\",\""))
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
        (AnyPredicate, Tag(tag)) => format!("null,\"{:s}\"", tag),
        (AnyPredicate, Tags(tags)) => format!("null,[\"{:s}\"]", tags.connect("\",\"")),

        (Predicate(predicate), AnyTag) => format!("\"{:s}\"", predicate),
        (Predicate(predicate), Tag(tag)) =>
            format!("\"{:s}\",\"{:s}\"", predicate, tag),
        (Predicate(predicate), Tags(tags)) =>
            format!("\"{:s}\",[\"{:s}\"]", predicate, tags.connect("\",\"")),

        (Predicates(predicates), AnyTag) =>
            format!("[\"{:s}\"]", predicates.connect("\",\"")),
        (Predicates(predicates), Tag(tag)) =>
            format!("[\"{:s}\"],\"{:s}\"", predicates.connect("\",\""), tag),
        (Predicates(predicates), Tags(tags)) =>
            format!("[\"{:s}\"],[\"{:s}\"]", predicates.connect("\",\""), tags.connect("\",\"")),

        (FromQuery(query), AnyTag) =>
            match query.compile() {
                Some((compiled, _)) => compiled,
                None => "null".to_string()
            },
        (FromQuery(query), Tag(tag)) =>
            format!("{:s}, \"{:s}\"",
                    match query.compile() {
                        Some((compiled, _)) => compiled,
                        None => "null".to_string()
                    },
                    tag),
        (FromQuery(query), Tags(tags)) =>
            format!("{:s}, [\"{:s}\"]",
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
        (AnyPredicate, Node(node)) => format!("null,\"{:s}\"", node),
        (AnyPredicate, Nodes(nodes)) => format!("null,[\"{:s}\"]", nodes.connect("\",\"")),

        (Predicate(predicate), AnyNode) => format!("\"{:s}\"", predicate),
        (Predicate(predicate), Node(tag)) =>
            format!("\"{:s}\",\"{:s}\"", predicate, tag),
        (Predicate(predicate), Nodes(nodes)) =>
            format!("\"{:s}\",[\"{:s}\"]", predicate, nodes.connect("\",\"")),

        (Predicates(predicates), AnyNode) =>
            format!("[\"{:s}\"]", predicates.connect("\",\"")),
        (Predicates(predicates), Node(node)) =>
            format!("[\"{:s}\"],\"{:s}\"", predicates.connect("\",\""), node),
        (Predicates(predicates), Nodes(nodes)) =>
            format!("[\"{:s}\"],[\"{:s}\"]", predicates.connect("\",\""), nodes.connect("\",\"")),

        (FromQuery(query), AnyNode) =>
            match query.compile() {
                Some((compiled, _)) => compiled,
                None => "null".to_string()
            },
        (FromQuery(query), Node(node)) =>
            format!("{:s},\"{:s}\"",
                    match query.compile() {
                        Some((compiled, _)) => compiled,
                        None => "null".to_string()
                    },
                    node),
        (FromQuery(query), Nodes(nodes)) =>
            format!("{:s},[\"{:s}\"]",
                    match query.compile() {
                        Some((compiled, _)) => compiled,
                        None => "null".to_string()
                    },
                    nodes.connect("\",\""))

    }
}

/*

#![feature(macro_rules)]

enum EnumOne {
    Var11,
    Var12,
    Var13
}

enum EnumTwo {
    Var21,
    Var22,
    Var23
}

macro_rules! enum_macro(
    [ $($e1:ident)->+ => $e2:ident ] => (
        vec!($($e1,)+), $e2
    )
)


fn with_enums(e1: Vec<EnumOne>, e2: EnumTwo) { }

fn main() {
    with_enums(enum_macro![Var11 -> Var12 => Var22 ]);
}

*/

/*
pub enum Test<'t> {
    TestVal(|int|:'t -> int),
    AnotherVal
}

fn ret_val<'t>() -> Test<'t> {
    TestVal(|x| { x * 2 })
}

fn main() {
    match ret_val() {
        TestVal(cl) => { println!("{:i}", cl(5)) },
        AnotherVal => {}
    }
}
 */
