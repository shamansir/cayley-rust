use selector::{NodeSelector, TagSelector, PredicateSelector};

use selector::{AnyNode, Node, Nodes};
use selector::{AnyTag, Tag, Tags};
use selector::{AnyPredicate, Predicate, Predicates};
use selector::Query as FromQuery;

#[macro_export]
macro_rules! vertex(
    [ $e1:ident -> $($e2:ident)->+ => $e3:ident ] => (
        Vertex($e1, &[$($e2,)+], $e3)
    )
)
/* macro_rules! enum_macro(
    [ $($e1:ident)->+ => $e2:ident ] => (
        vec!($($e1,)+), $e2
    )
) */

#[macro_export]
macro_rules! morphism(
    [ $($e1:ident)->+ => $e2:ident ] => (
        Morphism("", &[$($e1,)+])
    )
)

pub enum Path {
    Out(PredicateSelector, TagSelector),
    OutP(PredicateSelector)
    OutT(TagSelector),
    In(PredicateSelector, TagSelector),
    InP(PredicateSelector)
    InT(TagSelector),
    Both(PredicateSelector, TagSelector),
    BothP(PredicateSelector)
    BothT(TagSelector),
    Is(NodeSelector),
    Has(PredicateSelector, NodeSelector),
    Tag(TagSelector),
    As(TagSelector),
    Back(TagSelector),
    Save(PredicateSelector, TagSelector),
    Intersect(Query),
    And(Query),
    Union(Query),
    Or(Query),
    Follow(Reuse),
    FollowR(Reuse)
}

pub enum Query {
    All,
    GetLimit(int),
    ToArray,
    ToValue,
    TagArray,
    TagValue
    /* ForEach(|int|:'q -> int) */
}

/* pub enum Expectation {
    Unknown,
    SingleNode,
    NodeSequence,
    NameSequence,
    TagSequence,
    SingleTag
} */

pub struct Vertex(NodeSelector, &[Path], Query);
pub struct Morphism(&str, &[Path]);

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
