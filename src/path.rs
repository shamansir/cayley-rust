#![macro_escape]

use std::fmt::{Show, Formatter};
use std::fmt::Error as FormatError;

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
        match Traversals::compile_path(box [$e1, $($e2,)*]) {
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
    Intersect(&'t CompiledRoute),
    And(&'t CompiledRoute),
    Union(&'t CompiledRoute),
    Or(&'t CompiledRoute),
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

// ================================ Traversals ============================= //

pub struct Traversals<'ps>(pub Box<[Traversal<'ps>]>);

impl<'t> Traversals<'t> {

    pub fn compile_path<'a>(traversals: Box<[Traversal<'a>]>) -> Option<CompiledPath> {
        Traversals(traversals).compile_path()
    }

}

impl<'ts> ToString for Traversals<'ts> {

    fn to_string(&self) -> String {
        match self.compile_path() {
            Some(path) => path.value,
            None => "<Traversals: Incorrect>".to_string()
        }
    }

}

impl<'p> Path for Traversals<'p> {

    fn compile_path(&self) -> Option<CompiledPath> {
        match *self {
            Traversals(ref traversals) =>
                Some(CompiledPath {
                    prefix: parse_prefix(traversals),
                    value: parse_traversals(traversals)
                })
        }
    }

}

// ================================ Morphism ================================ //

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
                                                              &Nodes(ref names) => format!(".Is(\"{}\")", names.connect("\",\""))
                                                          },
            Traversal::Has(ref predicates, ref nodes)  => format!(".Has({})", parse_predicates_and_nodes(predicates, nodes)),
            // Tagging =========================================================================================================
            Traversal::Tag(ref tags) |
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
