#![feature(globs)]
#![feature(phase, macro_rules)]

#[phase(plugin, link)] extern crate cayley;

use cayley::path::*;

use cayley::selector::*;

macro_rules! path_eq(
    ($src:expr, $res:expr) => ( assert_eq!($src.prefix + $src.value, $res.to_string()); );
)

// Examples from: https://github.com/google/cayley/blob/master/docs/GremlinAPI.md

// NB: Queries without a Final (Paths) can be compiled and used in other Queries,
//     but not executed on graph. So Vertices may have Final or not.
//     Morphism always has no Final.

// == Vertex ==

#[test]
fn test_basic_vertices() {

    path_eq!(vertex![ AnyNode ], "g.V()");

    path_eq!(vertex![ Node("foo") ], "g.V(\"foo\")");

    path_eq!(vertex![ Nodes(vec!("foo", "bar")) ], "g.V(\"foo\",\"bar\")");

    path_eq!(vertex![ AnyNode => All ], "g.V().All()");

    path_eq!(vertex![ Node("foo") => All ], "g.V(\"foo\").All()");

    path_eq!(vertex![ Nodes(vec!("foo", "bar")) => All ], "g.V(\"foo\",\"bar\").All()");

    path_eq!(vertex![ Nodes(vec!("foo", "bar")) -> Is(Node("foo")) => All ],
             "g.V(\"foo\",\"bar\").Is(\"foo\").All()");

}

// == Morphism ==

#[test]
fn test_basic_morphism() {

    path_eq!(morphism![ "foobar" -> Out(Predicate("foo"), AnyTag)
                                 -> Out(Predicate("bar"), AnyTag) ],
             "g.M().Out(\"foo\").Out(\"bar\")");

    path_eq!(morphism![ "foobar" -> Out(Predicate("foo"), Tags(vec!("tag1", "tag2")))
                                 -> Out(Predicate("bar"), Tag("tag0")) ],
             "g.M().Out(\"foo\",[\"tag1\",\"tag2\"]).Out(\"bar\",\"tag0\")");

}

// == Writing ==

#[test]
fn test_writing() { /* TODO */ }

// == Emit ==

#[test]
fn test_emit() { /* TODO */ }

// == Basic Traversals ==

/* path.Out */

#[test]
fn test_path_out() {

    path_eq!(vertex![ Node("C") -> Out(Predicate("follows"), AnyTag) ],
             "g.V(\"C\").Out(\"follows\")");

    path_eq!(vertex![ Node("A") -> Out(Predicate("follows"), AnyTag)
                                -> Out(Predicate("follows"), AnyTag) ],
            "g.V(\"A\").Out(\"follows\").Out(\"follows\")");

    path_eq!(vertex![ Node("D") -> Out(AnyPredicate, AnyTag) ],
             "g.V(\"D\").Out()");

    path_eq!(vertex![ Node("D") -> Out(Predicates(vec!("follows", "status")), AnyTag) ],
             "g.V(\"D\").Out([\"follows\",\"status\"])");

    path_eq!(vertex![ Node("D") -> Out(Route(&vertex![ Node("status") ]), Tag("pred")) ],
             "g.V(\"D\").Out(g.V(\"status\"), \"pred\")");

    let use_twice = vertex![ Node("status") -> OutP(Predicate("foo")) ];

    path_eq!(vertex![ Node("E") -> Out(Route(&use_twice), Tag("next")) ],
             "g.V(\"E\").Out(g.V(\"status\").Out(\"foo\"), \"next\")");

    path_eq!(vertex![ Node("E") -> Out(Route(&use_twice), Tag("prev")) ],
             "g.V(\"E\").Out(g.V(\"status\").Out(\"foo\"), \"prev\")");

}

/* path.In */

#[test]
fn test_path_in() {

    path_eq!(vertex![ Node("cool_person") -> In(Predicate("status"), AnyTag) ],
             "g.V(\"cool_person\").In(\"status\")");

    path_eq!(vertex![ Node("B") -> In(Predicate("follows"), AnyTag) ],
             "g.V(\"B\").In(\"follows\")");

    path_eq!(vertex![ Node("E") -> Out(Predicate("follows"), AnyTag)
                                -> In(Predicate("follows"), AnyTag) ],
             "g.V(\"E\").Out(\"follows\").In(\"follows\")");

    /* TODO: test with tags names & arrays */
}

/* path.Both */

#[test]
fn test_path_both() {

    path_eq!(vertex![ Node("F") -> Both(Predicate("follows"), AnyTag) ],
             "g.V(\"F\").Both(\"follows\")");

}

/* path.Is */

#[test]
fn test_path_is() {

    path_eq!(vertex![ AnyNode -> Out(Predicate("follows"), AnyTag)
                              -> Is(Node("B")) ],
             "g.V().Out(\"follows\").Is(\"B\")");

    path_eq!(vertex![ AnyNode -> Out(Predicate("follows"), AnyTag)
                              -> Is(Nodes(vec!("B", "C"))) ],
             "g.V().Out(\"follows\").Is(\"B\",\"C\")");

}

/* path.Has */

#[test]
fn test_path_has() {

    path_eq!(vertex![ AnyNode -> Has(Predicate("follows"), Node("B")) ],
             "g.V().Has(\"follows\",\"B\")");

}

// == Tagging ==

/* path.Tag / path.As */

#[test]
fn test_path_tag_as() {

    path_eq!(vertex![ AnyNode -> As(Tag("start")) -> Out(Predicate("status"), AnyTag) ],
             "g.V().As(\"start\").Out(\"status\")");

    path_eq!(vertex![ AnyNode -> Traversal::Tag(Tags(vec!("foo", "bar")))
                              -> Out(Predicate("status"), AnyTag) ],
             "g.V().As(\"foo\",\"bar\").Out(\"status\")");

}

/* path.Back */

#[test]
fn test_path_back() {

    path_eq!(vertex![ AnyNode -> As(Tag("start"))
                              -> Out(Predicate("status"), AnyTag)
                              -> Back(Tag("start"))
                              -> In(Predicate("follows"), AnyTag) ],
             "g.V().As(\"start\").Out(\"status\").Back(\"start\").In(\"follows\")");

}

/* path.Save */

#[test]
fn test_path_save() {

    path_eq!(vertex![ Nodes(vec!("D", "B")) -> Save(Predicate("follows"), Tag("target")) ],
             "g.V(\"D\",\"B\").Save(\"follows\",\"target\")");

    /* TODO:
    path_panic!(V::start(Nodes(vec!("D", "B"))).Save(AnyPredicate, Tag("target")),
                "should fail to compile path.Save w/AnyPredicate");
    path_panic!(V::start(Nodes(vec!("D", "B"))).Save(Predicates(vec!("foo", "bar")), Tag("target")),
                "should fail to compile path.Save w/Predicates");
    path_panic!(V::start(Nodes(vec!("D", "B"))).Save(Predicate("follows"), AnyTag),
                "should fail to compile path.Save w/AnyTag");
    path_panic!(V::start(Nodes(vec!("D", "B"))).Save(Predicate("follows"), Tags(vec!("foo", "bar"))),
                "should fail to compile path.Save w/AnyTag"); */
}

// == Joining ==

/* path.Intersect / path.And */

#[test]
fn test_path_intersect_and() {

    let cFollows = vertex![ Node("C") -> Out(Predicate("follows"), AnyTag) ];
    let dFollows = vertex![ Node("D") -> Out(Predicate("follows"), AnyTag) ];

    path_eq!(cFollows + path![ Intersect(&dFollows) ],
             "g.V(\"C\").Out(\"follows\").And(g.V(\"D\").Out(\"follows\"))");
    path_eq!(cFollows + path![ And(&dFollows) ],
             "g.V(\"C\").Out(\"follows\").And(g.V(\"D\").Out(\"follows\"))");

}

/* path.Union / path.Or */

#[test]
fn test_path_union_or() {

    let cFollows = vertex![ Node("C") -> Out(Predicate("follows"), AnyTag) ];
    let dFollows = vertex![ Node("D") -> Out(Predicate("follows"), AnyTag) ];

    path_eq!(cFollows + path![ Union(&dFollows) ],
             "g.V(\"C\").Out(\"follows\").Or(g.V(\"D\").Out(\"follows\"))");
    path_eq!(cFollows + path![ Or(&dFollows) ],
             "g.V(\"C\").Out(\"follows\").Or(g.V(\"D\").Out(\"follows\"))");

}

/* fn test_concatenating_paths() {
    // TODO
} */

// == Morphisms ==

/* path.Follow */

#[test]
fn test_path_follow() {

    let friendOfFriend = morphism![ "friendOfFriend" -> Out(Predicate("follows"), AnyTag)
                                                     -> Out(Predicate("follows"), AnyTag) ];
    path_eq!(friendOfFriend, "g.M().Out(\"follows\").Out(\"follows\")");

    path_eq!(vertex![ Node("C") -> Follow(&friendOfFriend)
                                -> Has(Predicate("status"), Node("cool_person")) ],
             "var friendOfFriend = g.M().Out(\"follows\").Out(\"follows\");g.V(\"C\").Follow(friendOfFriend).Has(\"status\",\"cool_person\")");

}

/* path.FollowR */

#[test]
fn test_path_followr() {

    let friendOfFriend = morphism![ "friendOfFriend" -> Out(Predicate("follows"), AnyTag)
                                                     -> Out(Predicate("follows"), AnyTag) ];

    path_eq!(vertex![ AnyNode -> Has(Predicate("status"), Node("cool_person"))
                              -> FollowR(&friendOfFriend) ],
             "var friendOfFriend = g.M().Out(\"follows\").Out(\"follows\");g.V().Has(\"status\",\"cool_person\").FollowR(friendOfFriend)");

}

// == Query finals ==

#[test]
fn test_query_finals() {

    path_eq!(vertex![ AnyNode -> Out(Predicate("follows"), AnyTag) => All ],
             "g.V().Out(\"follows\").All()");

    path_eq!(vertex![ Node("foo") -> Out(Predicate("follows"), AnyTag) => GetLimit(5) ],
             "g.V(\"foo\").Out(\"follows\").GetLimit(5)");

}

// == Other ==

#[test]
fn test_inclusive_vertices() {

    let v_1 = vertex![ AnyNode -> Out(Predicate("follows"), AnyTag)
                               -> In(Predicate("follows"), AnyTag) ];
    let v_2 = vertex![ Node("bar") -> Has(Predicate("status"), Node("cool_person"))
                                   -> And(&v_1) ];
    path_eq!(vertex![ Node("foo") -> Union(&v_2) => All ],
             "g.V(\"foo\").Or(g.V(\"bar\").Has(\"status\",\"cool_person\").And(g.V().Out(\"follows\").In(\"follows\"))).All()")

    path_eq!(vertex![ Node("foo") -> Or(&v_2) ],
             "g.V(\"foo\").Or(g.V(\"bar\").Has(\"status\",\"cool_person\").And(g.V().Out(\"follows\").In(\"follows\")))")

}

#[test]
fn test_inclusive_moprphisms() {

    let m_1 = morphism![ "m1" -> Out(Predicate("follows"), AnyTag)
                              -> Out(Predicate("follows"), AnyTag) ];
    let m_2 = morphism![ "m2" -> Has(Predicate("status"), Node("cool_person"))
                              -> FollowR(&m_1) ];

    path_eq!(vertex![ Node("foo") -> Follow(&m_2) => All ],
             "var m1 = g.M().Out(\"follows\").Out(\"follows\");var m2 = g.M().Has(\"status\",\"cool_person\").FollowR(m1);g.V(\"foo\").Follow(m2).All()");

    path_eq!(vertex![ Node("foo") -> Follow(&m_2) ],
             "var m1 = g.M().Out(\"follows\").Out(\"follows\");var m2 = g.M().Has(\"status\",\"cool_person\").FollowR(m1);g.V(\"foo\").Follow(m2)");

}

/* TODO:

path_eq!(V::start(Node("bar")).In(Predicate("follows"), AnyTag).ToArray(),
         "g.V(\"bar\").In(\"follows\").toArray()");

path_eq!(V::start(AnyNode).Out(Predicate("follows"), AnyTag).ToValue(),
         "g.V().Out(\"follows\").ToValue()");

path_eq!(V::start(Node("foo")).Out(Predicate("follows"), AnyTag).TagValue(),
         "g.V(\"foo\").Out(\"follows\").TagValue()");

query.ForEach(callback), query.ForEach(limit, callback); */

/* TODO

// Let's get the list of actors in the film
g.V().Has("name","Casablanca")
     .Out("/film/film/starring").Out("/film/performance/actor")
     .Out("name").All()

// But this is starting to get long. Let's use a morphism -- a pre-defined path stored in a variable -- as our linkage

var filmToActor = g.Morphism().Out("/film/film/starring").Out("/film/performance/actor")

g.V().Has("name", "Casablanca").Follow(filmToActor).Out("name").All() */

fn main() { }
