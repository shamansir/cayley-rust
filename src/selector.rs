use path;

pub enum NodeSelector<'ns> {
    AnyNode,
    Node(&'ns str),
    Nodes(Vec<&'ns str>)
}

pub enum PredicateSelector<'ps> {
    AnyPredicate,
    Predicate(&'ps str),
    Predicates(Vec<&'ps str>),
    Route(&'ps path::CompiledRoute)
}

pub enum TagSelector<'ts> {
    AnyTag,
    Tag(&'ts str),
    Tags(Vec<&'ts str>)
}
