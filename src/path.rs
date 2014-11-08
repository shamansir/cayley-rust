use selector::{NodeSelector, TagSelector, PredicateSelector};

use selector::{AnyNode, Node, Nodes};
use selector::{AnyTag, Tag, Tags};
use selector::{AnyPredicate, Predicate, Predicates};
use selector::Query as FromQuery;

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
    Follow(&Reuse),
    FollowR(&Reuse)
}

pub enum Query {
    All,
    GetLimit(int),
    ToArray,
    ToValue,
    TagArray,
    TagValue,
    /* ForEach */
}

/* pub enum Expectation {
    Unknown,
    SingleNode,
    NodeSequence,
    NameSequence,
    TagSequence,
    SingleTag
} */


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
