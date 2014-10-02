use selector::{NodeSelector, TagSelector, PredicateSelector};

use selector::{AnyNode, Node, Nodes};
use selector::{AnyTag, Tag, Tags};
use selector::{AnyPredicate, Predicate, Predicates, FromQuery};

pub struct Vertex {
    finalized: bool,
    path: Vec<String>
}

pub struct Morphism {
    name: String,
    path: Vec<String>
}

pub trait AddString {

    fn add_str(&mut self, str: &str) -> &Self;

    fn add_string(&mut self, str: String) -> &Self;

}

// FIXME: may conflict with std::Path
#[allow(non_snake_case)]
pub trait Path: AddString/*+ToString*/ {

    fn compile(&self) -> Option<String>;

    /* fn to_string(&self) -> String {
        match self.compile() {
            Some(compiled) => compiled,
            None => "[-]".to_string()
        }
    }*/

    fn Out(&mut self, predicates: PredicateSelector, tags: TagSelector) -> &Self {
        self.add_string(format!("Out({:s})", make_args_from(predicates, tags)))
    }

    // TODO: in, both...

}

#[allow(non_snake_case)]
pub trait Query: Path {

    fn set_finalized(&mut self);

    fn is_finalized(&self) -> bool;

    fn All(&mut self) -> &Self { self.set_finalized(); self.add_str("All()") }

    // TODO: get_limit....

}

impl Vertex {

    pub fn start(nodes: NodeSelector) -> Vertex {
        let mut res = Vertex{ path: Vec::with_capacity(10), finalized: false };
        res.add_str("graph");
        res.add_string(match nodes {
                Nodes(names) => format!("Vertex(\"{:s}\")", names.connect(",")),
                Node(name) => format!("Vertex(\"{:s}\")", name),
                AnyNode/*| Node("") */ => "Vertex()".to_string()
            });
        res
    }

}

impl AddString for Vertex {

    fn add_str(&mut self, str: &str) -> &Vertex {
        self.path.push(str.to_string());
        self
    }

    fn add_string(&mut self, str: String) -> &Vertex {
        self.path.push(str);
        self
    }

}

impl Path for Vertex {

    fn compile(&self) -> Option<String> {
        // a bolt-hole to return None, if path was incorrectly constructed
        Some(self.path.connect("."))
    }

}

impl Query for Vertex {

    fn set_finalized(&mut self) { self.finalized = true; }

    fn is_finalized(&self) -> bool { self.finalized }

}

impl Morphism {

    pub fn start(name: &str) -> Morphism {
        let mut res = Morphism { name: name.to_string(), path: Vec::with_capacity(10) };
        res.add_string(name.to_string() + " = graph.Morphism()".to_string());
        res
    }

}

impl AddString for Morphism {

    fn add_str(&mut self, str: &str) -> &Morphism {
        self.path.push(str.to_string());
        self
    }

    fn add_string(&mut self, str: String) -> &Morphism {
        self.path.push(str);
        self
    }

}

impl Path for Morphism {

    fn compile(&self) -> Option<String> {
        // a bolt-hole to return None, if path was incorrectly constructed
        Some(self.path.connect("."))
    }

}


fn make_args_from(predicates: PredicateSelector, tags: TagSelector) -> String {
    match (predicates, tags) {

        (AnyPredicate, AnyTag) => "".to_string(),
        (AnyPredicate, Tag(tag)) => format!("null, \"{:s}\"", tag),
        (AnyPredicate, Tags(tags)) => format!("null, \"{:s}\"", tags.connect("\",\"")),

        (Predicate(predicate), AnyTag) => format!("\"{:s}\"", predicate),
        (Predicate(predicate), Tag(tag)) =>
            format!("\"{:s}\", \"{:s}\"", predicate, tag),
        (Predicate(predicate), Tags(tags)) =>
            format!("\"{:s}\", \"{:s}\"", predicate, tags.connect("\",\"")),

        (Predicates(predicates), AnyTag) =>
            format!("\"{:s}\"", predicates.connect("\",\"")),
        (Predicates(predicates), Tag(tag)) =>
            format!("\"{:s}\", \"{:s}\"", predicates.connect("\",\""), tag),
        (Predicates(predicates), Tags(tags)) =>
            format!("\"{:s}\", \"{:s}\"", predicates.connect("\",\""), tags.connect("\",\"")),

        (FromQuery(query), AnyTag) =>
            match query.compile() {
                Some(compiled) => compiled,
                None => "undefined".to_string()
            },
        (FromQuery(query), Tag(tag)) =>
            format!("{:s}, \"{:s}\"",
                    match query.compile() {
                        Some(compiled) => compiled,
                        None => "undefined".to_string()
                    },
                    tag),
        (FromQuery(query), Tags(tags)) =>
            format!("{:s}, \"{:s}\"",
                    match query.compile() {
                        Some(compiled) => compiled,
                        None => "undefined".to_string()
                    },
                    tags.connect("\",\""))

    }
}
