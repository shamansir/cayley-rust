use path::Query;

pub enum NodeSelector<'ns> {
    AnyNode,
    Node(&'ns str),
    Nodes(Vec<&'ns str>)
}

pub enum PredicateSelector<'ps> {
    AnyPredicate,
    Predicate(&'ps str),
    Predicates(Vec<&'ps str>),
    FromQuery(Box<Query+'ps>)
}

pub enum TagSelector<'ts> {
    AnyTag,
    Tag(&'ts str),
    Tags(Vec<&'ts str>)
}
