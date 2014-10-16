use selector::{NodeSelector, TagSelector, PredicateSelector};

use selector::{AnyNode, Node, Nodes};
use selector::{AnyTag, Tag, Tags};
use selector::{AnyPredicate, Predicate, Predicates};
use selector::Query as FromQuery;

use std::fmt::{Show, Formatter, FormatError};

pub struct Vertex {
    finalized: bool,
    path: Vec<String>
}

pub struct Morphism {
    saved: bool,
    name: String,
    path: Vec<String>
}

// ================================ Compile ================================= //

pub trait Compile: Clone/*+ToString*/ {

    fn add_str(&mut self, what: &str) -> &mut Self;

    fn add_string(&mut self, what: String) -> &mut Self;

    fn compile(&self) -> Option<String>;

    /* fn to_string(&self) -> String {
        match self.compile() {
            Some(compiled) => compiled,
            None => "[-]".to_string()
        }
    }

    fn fmt(&self, fmt: &mut Formatter) -> Result<(), FormatError> {
        write!(fmt, "{}", self.to_string())
    } */

}

// ================================ Path ==================================== //

#[allow(non_snake_case)]
pub trait Path: Compile {

    fn Out(&mut self, predicates: PredicateSelector, tags: TagSelector) -> &mut Self {
        self.add_string(format!("Out({:s})", predicates_and_tags(predicates, tags)))
    }

    fn OutP(&mut self, predicates: PredicateSelector) -> &mut Self {
        self.Out(predicates, AnyTag)
    }

    fn OutT(&mut self, tags: TagSelector) -> &mut Self {
        self.Out(AnyPredicate, tags)
    }

    fn In(&mut self, predicates: PredicateSelector, tags: TagSelector) -> &mut Self {
        self.add_string(format!("In({:s})", predicates_and_tags(predicates, tags)))
    }

    fn InP(&mut self, predicates: PredicateSelector) -> &mut Self {
        self.In(predicates, AnyTag)
    }

    fn InT(&mut self, tags: TagSelector) -> &mut Self {
        self.In(AnyPredicate, tags)
    }

    fn Both(&mut self, predicates: PredicateSelector, tags: TagSelector) -> &mut Self {
        self.add_string(format!("Both({:s})", predicates_and_tags(predicates, tags)))
    }

    fn BothP(&mut self, predicates: PredicateSelector) -> &mut Self {
        self.Both(predicates, AnyTag)
    }

    fn BothT(&mut self, tags: TagSelector) -> &mut Self {
        self.Both(AnyPredicate, tags)
    }

    fn Is(&mut self, nodes: NodeSelector) -> &mut Self {
        self.add_string(match nodes {
            AnyNode/*| Node("") */ => "Is()".to_string(),
            Node(name) => format!("Is(\"{:s}\")", name),
            Nodes(names) => format!("Is(\"{:s}\")", names.connect("\",\""))
        })
    }

    fn Has(&mut self, predicates: PredicateSelector, nodes: NodeSelector) -> &mut Self {
        self.add_string(format!("Has({:s})", predicates_and_nodes(predicates, nodes)))
    }

    fn HasP(&mut self, predicates: PredicateSelector) -> &mut Self {
        self.Has(predicates, AnyNode)
    }

    fn HasN(&mut self, nodes: NodeSelector) -> &mut Self {
        self.Has(AnyPredicate, nodes)
    }

    fn Tag(&mut self, tags: TagSelector) -> &mut Self { self.As(tags) }

    fn As(&mut self, tags: TagSelector) -> &mut Self {
        self.add_string(match tags {
            AnyTag/*| Node("") */ => "As()".to_string(),
            Tag(name) => format!("As(\"{:s}\")", name),
            Tags(names) => format!("As(\"{:s}\")", names.connect("\",\""))
        })
    }

    fn Back(&mut self, tags: TagSelector) -> &mut Self {
        self.add_string(match tags {
            AnyTag/*| Node("") */ => "Back()".to_string(),
            Tag(name) => format!("Back(\"{:s}\")", name),
            Tags(names) => format!("Back(\"{:s}\")", names.connect("\",\""))
        })
    }

    fn Save(&mut self, predicates: PredicateSelector, tags: TagSelector) -> &mut Self {
        self.add_string(format!("Save({:s})", predicates_and_tags(predicates, tags)))
    }

    fn SaveP(&mut self, predicates: PredicateSelector) -> &mut Self {
        self.Save(predicates, AnyTag)
    }

    fn SaveT(&mut self, tags: TagSelector) -> &mut Self {
        self.Save(AnyPredicate, tags)
    }

    fn Intersect(&mut self, query: &Query) -> &mut Self { self.And(query) }

    fn And(&mut self, query: &Query) -> &mut Self {
        /* FIXME: implicit return looking not so good here? */
        match query.compile() {
            Some(compiled) => { return self.add_string(format!("And({:s})", compiled)); },
            None => { } /* FIXME: save error */
        }
        self
    }

    fn Union(&mut self, query: &Query) -> &mut Self { self.Or(query) }

    fn Or(&mut self, query: &Query) -> &mut Self {
        /* FIXME: implicit return looking not so good here? */
        match query.compile() {
            Some(compiled) => { return self.add_string(format!("Or({:s})", compiled)); },
            None => { } /* FIXME: save error */
        }
        self
    }

    fn Follow(&mut self, reusable: &Reuse) -> &mut Self {
        /* TODO: match reusable.is_saved() {
            notify that reusable may not be saved
        } */
        self.add_string(format!("Follow({:s})", reusable.get_name()))
    }

    fn FollowR(&mut self, reusable: &Reuse) -> &mut Self {
        /* TODO: match reusable.is_saved() {
            notify that reusable may not be saved
        } */
        self.add_string(format!("FollowR({:s})", reusable.get_name()))
    }

}

// ================================ Query =================================== //

#[allow(non_snake_case)]
pub trait Query: Path {

    fn set_finalized(&mut self);

    fn is_finalized(&self) -> bool;

    fn All(&mut self) -> &mut Self { self.set_finalized(); self.add_str("All()") }

    fn GetLimit(&mut self, limit: int) -> &mut Self {
        self.set_finalized(); self.add_string(format!("GetLimit({:i})", limit))
    }

    /* TODO: ToArray() */
    /* TODO: ToValue() */
    /* TODO: TagArray() */
    /* TODO: TagValue() */
    /* TODO: query.ForEach(callback), query.ForEach(limit, callback) */

}

// ================================ Vertex ================================== //

impl Vertex {

    pub fn start(nodes: NodeSelector) -> Vertex {
        let mut res = Vertex::prepare();
        res.From(nodes);
        res
    }

    /* FIXME: calling this with no From call afterwars should fail the construction */
    pub fn prepare() -> Vertex {
        Vertex{ path: Vec::with_capacity(10), finalized: false }
    }

    pub fn From(&mut self, nodes: NodeSelector) -> &mut Vertex {
        match self.path.is_empty() {
            true => (),
            false => fail!("Vertex.From should be the first method to be called after Vertex::prepare()
                           or Vertex::start(nodes) should be used instead")
        }
        self.add_str("g");
        self.add_string(match nodes {
                Nodes(names) => format!("V(\"{:s}\")", names.connect("\",\"")),
                Node(name) => format!("V(\"{:s}\")", name),
                AnyNode/*| Node("") */ => "V()".to_string()
            })
    }

}

impl Compile for Vertex {

    fn add_str(&mut self, str: &str) -> &mut Vertex {
        self.path.push(str.to_string());
        self
    }

    fn add_string(&mut self, str: String) -> &mut Vertex {
        self.path.push(str);
        self
    }

    fn compile(&self) -> Option<String> {
        // a bolt-hole to return None, if path was incorrectly constructed
        Some(self.path.connect("."))
    }

}

impl Path for Vertex { }

impl Query for Vertex {

    fn set_finalized(&mut self) { self.finalized = true; }

    fn is_finalized(&self) -> bool { self.finalized }

}

impl Clone for Vertex {

    fn clone(&self) -> Vertex {
        Vertex { finalized: self.finalized,
                 path: self.path.clone() }
    }

}

// ================================ Reuse =================================== //

pub trait Reuse: Compile {

    fn get_name(&self) -> &str;

    fn set_saved(&mut self);

    fn is_saved(&self) -> bool;

    fn save(&self) -> Option<String> {
        match self.compile() {
            Some(compiled) => Some(self.get_name().to_string() + " = " + compiled),
            None => None
        }
    }

    fn save_as(&self, name: &str) -> Option<String> {
        match self.compile() {
            Some(compiled) => Some(name.to_string() + " = " + compiled),
            None => None
        }
    }

}

// ================================ Morphism ================================ //

impl Morphism {

    pub fn start(name: &str) -> Morphism {
        let mut res = Morphism { name: name.to_string(),
                                 path: Vec::with_capacity(10),
                                 saved: false };
        res.add_string("g.M()".to_string());
        res
    }

}

impl Compile for Morphism {

    fn add_str(&mut self, str: &str) -> &mut Morphism {
        self.path.push(str.to_string());
        self
    }

    fn add_string(&mut self, str: String) -> &mut Morphism {
        self.path.push(str);
        self
    }

    fn compile(&self) -> Option<String> {
        // a bolt-hole to return None, if path was incorrectly constructed
        Some(self.path.connect("."))
    }

}

impl Path for Morphism { }

impl Reuse for Morphism {

    fn get_name(&self) -> &str { self.name.as_slice() }

    fn set_saved(&mut self) { self.saved = true; }

    fn is_saved(&self) -> bool { self.saved }

}

impl Clone for Morphism {

    fn clone(&self) -> Morphism {
        Morphism { saved: self.saved,
                   name: self.name.clone(),
                   path: self.path.clone() }
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
                Some(compiled) => compiled,
                None => "null".to_string()
            },
        (FromQuery(query), Tag(tag)) =>
            format!("{:s}, \"{:s}\"",
                    match query.compile() {
                        Some(compiled) => compiled,
                        None => "null".to_string()
                    },
                    tag),
        (FromQuery(query), Tags(tags)) =>
            format!("{:s}, [\"{:s}\"]",
                    match query.compile() {
                        Some(compiled) => compiled,
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
                Some(compiled) => compiled,
                None => "null".to_string()
            },
        (FromQuery(query), Node(node)) =>
            format!("{:s},\"{:s}\"",
                    match query.compile() {
                        Some(compiled) => compiled,
                        None => "null".to_string()
                    },
                    node),
        (FromQuery(query), Nodes(nodes)) =>
            format!("{:s},[\"{:s}\"]",
                    match query.compile() {
                        Some(compiled) => compiled,
                        None => "null".to_string()
                    },
                    nodes.connect("\",\""))

    }
}
