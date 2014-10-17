use selector::{NodeSelector, TagSelector, PredicateSelector};

use selector::{AnyNode, Node, Nodes};
use selector::{AnyTag, Tag, Tags};
use selector::{AnyPredicate, Predicate, Predicates};
use selector::Query as FromQuery;

/// An interface to a Path with the ability to be executed as a query to a database.
/// The main entry point to ask for [GraphNodes](./struct.GraphNodes.html) from database using [Graph](./struct.Graph.html) as an interceptor.
///
/// To query for anything you might describe with Path from database, use this pattern:
/// `graph.find(Vertex::start(<NodeSelector>).<PathMethod>().<PathMethod>(<method_arg>).....<QueryMethod>())`.
///
/// Example:
///
/// ```
/// graph.find(Vertex::start(AnyNode).As(Tags(vec!("tag-a", "tag-b"))).OutP("follows").All())`.
/// ```
///
/// Another example:
///
/// ```
/// use cayley::{Graph, DefaultVersion};
/// use cayley::GraphNodes;
/// use cayley::path::{Vertex, Query}; // Query trait import is required
/// use cayley::selector::AnyNode;
///
/// let graph = Graph::new("localhost", 64210, DefaultVersion).unwrap();
/// match graph.find(Vertex::start(AnyNode).All()) {
///    Ok(GraphNodes(nodes)) => assert!(nodes.len() > 0),
///    Err(error) => fail!(error.to_string()),
/// };
/// ```
///
/// Sometimes it is wanted to separate a vertex instance from a query construction.
/// Use `prepare` static method for this purpose, but then ensure to start a query with `From` call:
///
/// ```
/// let v = Vertex::prepare();
/// // ...
/// v.From(Node("C")).OutP("follows").GetLimit(5);
/// let other_v = Vertex.Node("D").Union(&v).All();
/// graph.find(other_v);
/// ```
pub struct Vertex {
    finalized: bool,
    path: Vec<String>
}

/// An interface to a Path with the ability to be saved and reused to
/// construct other Paths, but not to query anything.
///
/// Use it to prepare a Path and re-use it several times
///
/// ```
/// #![allow(unused_result)]
/// use cayley::{Graph, DefaultVersion};
/// use cayley::path::Vertex as V;
/// use cayley::path::Morphism as M;
/// use cayley::path::{Path, Query}; // both traits imports are required
/// use cayley::selector::{Predicate, Node};
///
/// let graph = Graph::new("localhost", 64210, DefaultVersion).unwrap();
/// let mut follows_m = M::start("foo");
///         follows_m.OutP(Predicate("follows"));
/// graph.save(&mut follows_m).unwrap();
/// graph.find(V::start(Node("C"))
///              .Follow(&follows_m)
///              .Has(Predicate("status"), Node("cool_person"))
///              .All()).unwrap();
/// ```
pub struct Morphism {
    saved: bool,
    name: String,
    path: Vec<String>
}

// ================================ Compile ================================= //

/// Marks any Path which is able to be compiled to a string Gremlin-compatible query
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

/// The trait which covers all the methods from [Gremlin API](https://github.com/google/cayley/blob/master/docs/GremlinAPI.md)
/// Path specification, but in a Rust way.
///
/// Some methods requiring two parameters like predicate and tags have a siblings to help you in the
/// cases when you need only one, like `Out(<Predicate>, <Tag>)` has a sibling `OutP(<Predicate>)`
/// (alias for `Out(<Predicate>, AnyTag)` and a sibling `OutT(<Tag>)` (alias for `Out(AnyPredicate, <Tag>)`.
///
/// The rules of conversion are like that:
///
/// For `.Out`, `.In`, `.Both`, `.Save` methods, using `.Out` as an example:
///
/// * `.Out(AnyPredicate, AnyTag)` is equivalent to Gremlin `.Out()`;
/// * `.Out(Predicate("foo"), AnyTag)` is equivalent to Gremlin `.Out("foo")`;
/// * `.Out(Predicate("foo"), Tag("bar"))` is equivalent to Gremlin `.Out("foo", "bar")`;
/// * `.Out(Predicates(vec!("foo", "bar")), AnyTag)` is equivalent to Gremlin `.Out(["foo", "bar"])`;
/// * `.Out(AnyPredicate, Tag("foo"))` is equivalent to Gremlin `.Out(null, "foo")`;
/// * `.Out(AnyPredicate, Tags(vec!("foo", "bar")))` is equivalent to Gremlin `.Out(null, ["foo", "bar"])`;
/// * `.Out(Predicates(vec!("foo", "bar")), Tags(vec!("bar", "foo")))` is equivalent to Gremlin `.Out(["foo", "bar"], ["bar", "foo"])`;
///
/// For `.OutP`, `.InP`, `.BothP`, `.SaveP` methods, using `.OutP` as an example:
///
/// * `.OutP(AnyPredicate)` is equivalent to Gremlin `.Out()`;
/// * `.OutP(Predicate("foo"))` is equivalent to Gremlin `.Out("foo")`;
/// * `.OutP(Predicates(vec!("foo", "bar")))` is equivalent to Gremlin `.Out(["foo", "bar"])`;
///
/// For `.OutT`, `.InT`, `.BothT`, `.SaveT` methods, using `.OutT` as an example:
///
/// * `.OutT(AnyTag)` is equivalent to Gremlin `.Out()`;
/// * `.OutT(Tag("foo"))` is equivalent to Gremlin `.Out(null, "foo")`;
/// * `.OutT(Tags(vec!("foo", "bar")))` is equivalent to Gremlin `.Out(null, ["foo", "bar"])`;
///
/// For `.Has`, `.HasP`, `.HasN` methods it is the same as for three types above,
/// just replace `TagSelector` with `NodeSelector`
///
/// For `.Tag`, `.As`, `.Back` methods, using `.As` as an example:
///
/// * `.As(AnyTag)` is equivalent to Gremlin `.As()` (which has no sense, but allowed);
/// * `.As(Tag("foo"))` is equivalent to Gremlin `.As("foo")`;
/// * `.As(Tags(vec!("foo", "bar")))` is equivalent to Gremlin `.As("foo", "bar")`;
///
/// For `.Is` method:
///
/// * `.Is(AnyNode)` is equivalent to Gremlin `.Is()` (which has no sense, but allowed);
/// * `.Is(Node("foo"))` is equivalent to Gremlin `.Is("foo")`;
/// * `.Is(Nodes(vec!("foo", "bar")))` is equivalent to Gremlin `.Is("foo", "bar")`;
///
/// For `.Intersect`, `.And`, `.Union`, `.Or` methods, using `.Intersect` as example:
///
/// * `let some_v = Vertex(AnyNode).OutT(Tag("follows")).All();`
///   `graph.find(Vertex::start(Node("C")).Intersect(&some_v).All());`
///    is equivalent to Gremlin `g.V("C").Intersect(g.V.Out("follows").All()).All();`;
///
/// For `Follow` and `FollowR` methods:
///
/// * `let m = Morphism::start("foo")...;`
///   `graph.save(m);`
///   `graph.find(Vertex::start(AnyNode).Follow(&m).All());` is equivalent to Gremlin
///   `var foo = g.M()...; g.V().Follow(foo).All();`;
#[allow(non_snake_case)]
pub trait Path: Compile {

    /// `.Out` Path method. Follow forwards the quads with given predicates.
    fn Out(&mut self, predicates: PredicateSelector, tags: TagSelector) -> &mut Self {
        self.add_string(format!("Out({:s})", predicates_and_tags(predicates, tags)))
    }

    /// `OutP`, an alias to `Out(<Predicate>, AnyTag)`
    fn OutP(&mut self, predicates: PredicateSelector) -> &mut Self {
        self.Out(predicates, AnyTag)
    }

    /// `OutT`, an alias to `Out(AnyPredicate, <Tag>)`
    fn OutT(&mut self, tags: TagSelector) -> &mut Self {
        self.Out(AnyPredicate, tags)
    }

    /// `.In` Path method. Follow backwards the quads with given predicates.
    fn In(&mut self, predicates: PredicateSelector, tags: TagSelector) -> &mut Self {
        self.add_string(format!("In({:s})", predicates_and_tags(predicates, tags)))
    }

    /// `InP`, an alias to `In(<Predicate>, AnyTag)`
    fn InP(&mut self, predicates: PredicateSelector) -> &mut Self {
        self.In(predicates, AnyTag)
    }

    /// `InT`, an alias to `In(AnyPredicate, <Tag>)`
    fn InT(&mut self, tags: TagSelector) -> &mut Self {
        self.In(AnyPredicate, tags)
    }

    /// `.Both` Path method.
    fn Both(&mut self, predicates: PredicateSelector, tags: TagSelector) -> &mut Self {
        self.add_string(format!("Both({:s})", predicates_and_tags(predicates, tags)))
    }

    /// `BothP`, an alias to `Both(<Predicate>, AnyTag)`
    fn BothP(&mut self, predicates: PredicateSelector) -> &mut Self {
        self.Both(predicates, AnyTag)
    }

    /// `BothT`, an alias to `Both(AnyPredicate, <Tag>)`
    fn BothT(&mut self, tags: TagSelector) -> &mut Self {
        self.Both(AnyPredicate, tags)
    }

    /// `.Is` Path method. Filter all paths which are on the given node(-s).
    fn Is(&mut self, nodes: NodeSelector) -> &mut Self {
        self.add_string(match nodes {
            AnyNode/*| Node("") */ => "Is()".to_string(),
            Node(name) => format!("Is(\"{:s}\")", name),
            Nodes(names) => format!("Is(\"{:s}\")", names.connect("\",\""))
        })
    }

    /// `.Has` Path method. Filter all paths which are on the subject, but do not follow the path.
    fn Has(&mut self, predicates: PredicateSelector, nodes: NodeSelector) -> &mut Self {
        self.add_string(format!("Has({:s})", predicates_and_nodes(predicates, nodes)))
    }

    /// `HasP`, an alias to `Has(<Predicate>, AnyNode)`
    fn HasP(&mut self, predicates: PredicateSelector) -> &mut Self {
        self.Has(predicates, AnyNode)
    }

    /// `HasN`, an alias to `Has(AnyPredicate, <Node>)`
    fn HasN(&mut self, nodes: NodeSelector) -> &mut Self {
        self.Has(AnyPredicate, nodes)
    }

    /// `.Tag`, an alias to `.As`
    fn Tag(&mut self, tags: TagSelector) -> &mut Self { self.As(tags) }

    /// `.As` Path method. Mark items with a tag.
    fn As(&mut self, tags: TagSelector) -> &mut Self {
        self.add_string(match tags {
            AnyTag/*| Node("") */ => "As()".to_string(),
            Tag(name) => format!("As(\"{:s}\")", name),
            Tags(names) => format!("As(\"{:s}\")", names.connect("\",\""))
        })
    }

    /// `.Back` Path method. Follow backwards the tagged quads.
    fn Back(&mut self, tags: TagSelector) -> &mut Self {
        self.add_string(match tags {
            AnyTag/*| Node("") */ => "Back()".to_string(),
            Tag(name) => format!("Back(\"{:s}\")", name),
            Tags(names) => format!("Back(\"{:s}\")", names.connect("\",\""))
        })
    }

    /// `.Save` Path method. Save all quads with predicate into tag, without traversal.
    fn Save(&mut self, predicates: PredicateSelector, tags: TagSelector) -> &mut Self {
        self.add_string(format!("Save({:s})", predicates_and_tags(predicates, tags)))
    }

    /// `SaveP`, an alias to `Save(<Predicate>, AnyTag)`
    fn SaveP(&mut self, predicates: PredicateSelector) -> &mut Self {
        self.Save(predicates, AnyTag)
    }

    /// `SaveT`, an alias to `Save(AnyPredicate, <Tag>)`
    fn SaveT(&mut self, tags: TagSelector) -> &mut Self {
        self.Save(AnyPredicate, tags)
    }

    /// `.Intersect`, an alias to `.And`
    fn Intersect(&mut self, query: &Query) -> &mut Self { self.And(query) }

    fn And(&mut self, query: &Query) -> &mut Self {
        /* FIXME: implicit return looking not so good here? */
        match query.compile() {
            Some(compiled) => { return self.add_string(format!("And({:s})", compiled)); },
            None => { } /* FIXME: save error */
        }
        self
    }

    /// `.Union`, an alias to `.Or`
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

    /// `.All` Query method. Equivalent to Gremlin `Query.All()`.
    /// Returns all the items found within this path.
    fn All(&mut self) -> &mut Self { self.set_finalized(); self.add_str("All()") }

    /// `.GetLimit` Query method. Equivalent to Gremlin `Query.GetLimit(<number>)`.
    /// Returns first `<n>` items found within this path.
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

#[allow(non_snake_case)]
impl Vertex {

    /// Create a Vertex instance and start a query from [NodeSelector](./selector/struct.NodeSelector.html)
    pub fn start(nodes: NodeSelector) -> Vertex {
        let mut res = Vertex::prepare();
        res.From(nodes);
        res
    }

    /// Prepare a vertex instance to specify a query later. Ensure to start a query with `.From()` method
    /// if you use `prepare()`.
    pub fn prepare() -> Vertex {
        /* FIXME: calling this with no From call afterwars should fail the construction */
        Vertex{ path: Vec::with_capacity(10), finalized: false }
    }

    /// A method for postponed query creation, intended to be used after the `prepare()` method
    /// on the same Vertex instance.
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

    /// Get a prepared name for this Reusable
    pub fn get_name(&self) -> &str;

    fn set_saved(&mut self);

    /// Was this item saved at least once in _some_ graph during this session.
    pub fn is_saved(&self) -> bool;

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

    /// Create a Morphism instance with intention to store it in database under the given name
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
