#![macro_escape]

use std::fmt::{Show, Formatter};
use std::fmt::Error as FormatError;

use selector::{NodeSelector, TagSelector, PredicateSelector};

use selector::NodeSelector::{AnyNode, Node, Nodes};
use selector::TagSelector::{AnyTag, Tag, Tags};
use selector::PredicateSelector::{AnyPredicate, Predicate, Predicates};
use selector::PredicateSelector::Path as FromPath;

#[macro_export]
macro_rules! vertex(
    [ $e1:expr $(-> $e2:expr)* => $e3:expr ] => (
        match Vertex::compile($e1, box [$($e2,)*], $e3) {
            Some(v) => v, None => panic!("Vertex query failed to compile!")
        }
    );
    [ $e1:expr $(-> $e2:expr)+ ] => (
        match Vertex::compile_path($e1, box [$($e2,)*]) {
            Some(v) => v, None => panic!("Vertex query failed to compile!")
        }
    );
    [ $e1:expr ] => (
        match Vertex::compile_path($e1, box []) {
            Some(v) => v, None => panic!("Vertex query failed to compile!")
        }
    )
)

#[macro_export]
macro_rules! morphism(
    [ $e1:expr $(-> $e2:expr)+ ] => (
        match Morphism::compile($e1, box [$($e2,)*]) {
            Some(m) => m, None => panic!("Morphism path failed to compile!")
        }
    )
)

#[macro_export]
macro_rules! path(
    [ $e1:expr $(-> $e2:expr)* ] => (
        match Traversals::compile(box [$e1, $($e2,)*]) {
            Some(m) => m, None => panic!("Traversals path failed to compile!")
        }
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
    Intersect(&'t CompiledQuery),
    And(&'t CompiledQuery),
    Union(&'t CompiledQuery),
    Or(&'t CompiledQuery),
    // Morphisms
    Follow(&'t CompiledReuse),
    FollowR(&'t CompiledReuse)
}

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

pub enum Expectation {
    Unknown,
    SingleNode,
    NodeSequence,
    NameSequence,
    TagSequence,
    SingleTag
}

// ================================ Path, Query & Reuse ===================== //

trait Path: ToString {

    fn compile_path(&self) -> Option<String>;

}

trait Query: Path {

    fn compile_query(&self) -> Option<(String, Expectation)>;

}

trait Reuse: Path {

    fn get_name(&self) -> &str;

    fn compile_reuse(&self) -> Option<(&str, String)>;

}

/// Stores part of a path, i.e. `.Out("foo").Intersect(bar).Has("buz")`
pub struct PartialPath {
    pub value: String
}

/// Stores non-finalized path, i.e. `g.V().Out("foo").Intersect(bar).Has("buz")`
pub struct CompiledPath {
    pub value: String
}

/// Stores a query, i.e. `g.V().Out("foo").Intersect(bar).Has("buz").GetLimit(10)`
pub struct CompiledQuery {
    pub value: String,
    pub expectation: Expectation
}

/// Stores a named path, i.e. `g.M().Out("foo").Intersect(bar).Has("buz")` named as `"out_int_has"`
pub struct CompiledReuse {
    pub name: String,
    pub value: String
}

impl Add<PartialPath, PartialPath> for PartialPath {

    fn add(&self, _rhs: &PartialPath) -> PartialPath {
        PartialPath { value: self.value + _rhs.value }
    }

}

impl Add<PartialPath, CompiledPath> for CompiledPath {

    fn add(&self, _rhs: &PartialPath) -> CompiledPath {
        CompiledPath { value: self.value + _rhs.value }
    }

}

// ================================ Traversals ============================= //

pub struct Traversals<'ps>(pub Box<[Traversal<'ps>]>);

impl<'ps> Traversals<'ps> {

    pub fn compile<'a>(traversals: Box<[Traversal<'a>]>) -> Option<PartialPath> {
        match Traversals(traversals).compile_path() {
            Some(path) => Some(PartialPath { value: path }),
            None => None
        }
    }

}

impl<'p> Path for Traversals<'p> {

    fn compile_path(&self) -> Option<String> {
        match *self {
            Traversals(ref traversals) =>
                Some(parse_traversals(traversals))
        }
    }

}

impl<'ts> ToString for Traversals<'ts> {

    fn to_string(&self) -> String {
        match self.compile_path() {
            Some(path) => path,
            None => "<Traversals: Incorrect>".to_string()
        }
    }

}

// ================================ Morphism ================================ //

pub struct Morphism<'m>(pub &'m str, pub Box<[Traversal<'m>]>);

impl<'m> Morphism<'m> {

    pub fn compile<'a>(name: &'a str, traversals: Box<[Traversal<'a>]>) -> Option<CompiledReuse> {
        match Morphism(name, traversals).compile_reuse() {
            Some((name, path)) => Some(CompiledReuse {
                                           name: name.to_string(), value: path }),
            None => None
        }
    }

}

impl<'ts> ToString for Morphism<'ts> {

    fn to_string(&self) -> String {
        match self.compile_reuse() {
            Some((name, path)) => {
                name.to_string() + ":" + path
            },
            None => "<Morphism: Incorrect>".to_string()
        }
    }

}

impl<'p> Path for Morphism<'p> {

    fn compile_path(&self) -> Option<String> {
        match *self {
            Morphism(_, ref traversals) =>
                Some(parse_traversals(traversals))
        }
    }

}

impl<'r> Reuse for Morphism<'r> {

    fn get_name(&self) -> &str {
        match *self {
            Morphism(name, _) => name
        }
    }

    fn compile_reuse(&self) -> Option<(&str, String)> {
        match *self {
            Morphism(name, ref traversals) =>
                Some((name, "g.M()".to_string() + parse_traversals(traversals)))
        }
    }

}

// ================================ Vertex ================================== //

pub struct Vertex<'v>(pub NodeSelector<'v>, pub Box<[Traversal<'v>]>, pub Final);

impl<'v> Vertex<'v> {

    pub fn compile<'a>(start: NodeSelector<'a>, traversals: Box<[Traversal<'a>]>, _final: Final) -> Option<CompiledQuery> {
        match Vertex(start, traversals, _final).compile_query() {
            Some((query, expectation)) => Some(CompiledQuery {
                                                   value: query, expectation: expectation }),
            None => None
        }
    }

    pub fn compile_path<'a>(start: NodeSelector<'a>, traversals: Box<[Traversal<'a>]>) -> Option<CompiledPath> {
        match Vertex(start, traversals, Final::Undefined).compile_path() {
            Some(path) => Some(CompiledPath { value: path }),
            None => None
        }
    }

}

impl<'ts> ToString for Vertex<'ts> {

    fn to_string(&self) -> String {
        match self.compile_query() {
            Some((query, _)) => query,
            None => "<Vertex: Incorrect>".to_string()
        }

    }

}

impl<'p> Path for Vertex<'p> {

    fn compile_path(&self) -> Option<String> {
        match *self {
            Vertex(_, ref traversals, _) =>
                Some(parse_traversals(traversals))
        }
    }

}

impl<'q> Query for Vertex<'q> {

    fn compile_query(&self) -> Option<(String, Expectation)> {
        match *self {
            Vertex(ref start, ref traversals, _final) => {
                let mut result = String::new();
                for traversal in traversals.iter() {
                    match *traversal {
                        Traversal::Follow(reusable) | Traversal::FollowR(reusable) => {
                            result.push_str(format!("var {name} = {path};",
                                                    name = reusable.name,
                                                    path = reusable.value).as_slice());
                        }
                        _ => {}
                    }
                }
                result.push_str(parse_start(start).as_slice());
                result.push_str(parse_traversals(traversals).as_slice());
                match _final {
                    Final::Undefined => {},
                    _ => result.push_str(parse_final(_final).as_slice())
                }
                Some((result, match _final {
                    Final::Undefined    => Expectation::Unknown,
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

        (&FromPath(path), &AnyTag) => path.value.clone(),
        (&FromPath(path), &Tag(tag)) =>
            format!("{0}, \"{1}\"", path.value, tag),
        (&FromPath(path), &Tags(ref tags)) =>
            format!("{0}, [\"{1}\"]", path.value, tags.connect("\",\""))

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

        (&FromPath(path), &AnyNode) => path.value.clone(),
        (&FromPath(path), &Node(node)) =>
            format!("{0},\"{1}\"", path.value, node),
        (&FromPath(path), &Nodes(ref nodes)) =>
            format!("{0},[\"{1}\"]", path.value, nodes.connect("\",\""))

    }
}
