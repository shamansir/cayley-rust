#![feature(macro_rules)]

extern crate cayley;

use cayley::path::Vertex as V;
use cayley::path::Morphism as M;

// vote or discuss here:
// http://discuss.rust-lang.org/t/no-requirement-to-import-a-trait-for-using-an-implemented-public-method-from-it/579
use cayley::path::Compile; // required to use .compile() method
use cayley::path::Path; // required to be able to use Path methods such as .In, .Out, ...
use cayley::path::Query; // required to be able to use Query methods such as .All, .GetLimit, ...

use cayley::selector::{AnyNode, Node, Nodes};
use cayley::selector::{AnyTag, Tag, Tags};
use cayley::selector::{AnyPredicate, Predicate, Predicates, Query};

#[test]
#[allow(non_snake_case)]
fn main() {

    macro_rules! path_eq(
        ($src:expr, $res:expr) => (
            assert_eq!($src.compile(), Some($res.to_string()));
        );
    )

    macro_rules! path_fail(
        ($src:expr, $msg:ident) => (
            match $src.compile() {
                Some(_) => fail!($msg),
                None => ()
            };
        );
    )

    // Examples from: https://github.com/google/cayley/blob/master/docs/GremlinAPI.md

    // == Vertex ==

    // can be compiled, but not executed
    path_eq!(V::start(AnyNode), "g.V()");

    // can be compiled, but not executed
    path_eq!(V::start(Node("foo")), "g.V(\"foo\")");

    // can be compiled, but not executed
    path_eq!(V::start(Nodes(vec!("foo", "bar"))), "g.V(\"foo\",\"bar\")");

    path_eq!(V::start(AnyNode).All(), "g.V().All()");

    path_eq!(V::start(Node("foo")).All(), "g.V(\"foo\").All()");

    path_eq!(V::start(Nodes(vec!("foo", "bar"))).All(),
             "g.V(\"foo\",\"bar\").All()");

    path_eq!(V::start(Nodes(vec!("foo", "bar"))).All(),
             "g.V(\"foo\",\"bar\").All()");

    // == Morphism ==

    match M::start("morph").Out(Predicate("foo"), AnyTag)
                           .Out(Predicate("bar"), AnyTag).compile() {
        Some(result) => {
            assert_eq!(result, "g.M().Out(\"foo\").Out(\"bar\")".to_string())
        }
        None => fail!()
    }

    path_eq!(M::start("morph").Out(Predicate("foo"), AnyTag)
                              .Out(Predicate("bar"), AnyTag),
             "g.M().Out(\"foo\").Out(\"bar\")");

    path_eq!(M::start("morph").Out(Predicate("foo"), Tags(vec!("tag1", "tag2")))
                              .Out(Predicate("bar"), Tag("tag0")),
             "g.M().Out(\"foo\",[\"tag1\",\"tag2\"]).Out(\"bar\",\"tag0\")");

    /* TODO: test saving */

    // == Emit ==

    /* TODO: */

    // == Basic Traversals ==

    // path.Out

    path_eq!(V::start(Node("C")).Out(Predicate("follows"), AnyTag),
             "g.V(\"C\").Out(\"follows\")");

    path_eq!(V::start(Node("A")).Out(Predicate("follows"), AnyTag)
                                .Out(Predicate("follows"), AnyTag),
             "g.V(\"A\").Out(\"follows\").Out(\"follows\")");

    path_eq!(V::start(Node("D")).Out(AnyPredicate, AnyTag),
             "g.V(\"D\").Out()");

    path_eq!(V::start(Node("D")).Out(Predicates(vec!("follows", "status")), AnyTag),
             "g.V(\"D\").Out([\"follows\",\"status\"])");

    path_eq!(V::start(Node("D")).Out(Query(&V::start(Node("status"))), Tag("pred")),
             "g.V(\"D\").Out(g.V(\"status\"), \"pred\")");

    // path.In

    path_eq!(V::start(Node("cool_person")).In(Predicate("status"), AnyTag),
             "g.V(\"cool_person\").In(\"status\")");

    path_eq!(V::start(Node("B")).In(Predicate("follows"), AnyTag),
             "g.V(\"B\").In(\"follows\")");

    path_eq!(V::start(Node("E")).Out(Predicate("follows"), AnyTag)
                                .In(Predicate("follows"), AnyTag),
             "g.V(\"E\").Out(\"follows\").In(\"follows\")");

    /* TODO: test with tags names & arrays */

    // path.Both

    path_eq!(V::start(Node("F")).Both(Predicate("follows"), AnyTag),
             "g.V(\"F\").Both(\"follows\")");

    // path.Is

    path_eq!(V::start(AnyNode).Out(Predicate("follows"), AnyTag).Is(Node("B")),
             "g.V().Out(\"follows\").Is(\"B\")");

    path_eq!(V::start(AnyNode).Out(Predicate("follows"), AnyTag).Is(Nodes(vec!("B", "C"))),
             "g.V().Out(\"follows\").Is(\"B\",\"C\")");

    // path.Has

    path_eq!(V::start(AnyNode).Has(Predicate("follows"), Node("B")),
             "g.V().Has(\"follows\",\"B\")");

    // == Tagging ==

    // path.Tag / path.As

    path_eq!(V::start(AnyNode).As(Tag("start")).Out(Predicate("status"), AnyTag),
             "g.V().As(\"start\").Out(\"status\")");

    path_eq!(V::start(AnyNode).Tag(Tags(vec!("foo", "bar"))).Out(Predicate("status"), AnyTag),
             "g.V().As(\"foo\",\"bar\").Out(\"status\")");

    // path.Back

    path_eq!(V::start(AnyNode).As(Tag("start")).Out(Predicate("status"), AnyTag)
                              .Back(Tag("start")).In(Predicate("follows"), AnyTag),
             "g.V().As(\"start\").Out(\"status\").Back(\"start\").In(\"follows\")");

    // path.Save

    path_eq!(V::start(Nodes(vec!("D", "B"))).Save(Predicate("follows"), Tag("target")),
             "g.V(\"D\",\"B\").Save(\"follows\",\"target\")");

    /* TODO:
    path_fail!(V::start(Nodes(vec!("D", "B"))).Save(AnyPredicate, Tag("target")),
               "should fail to compile path.Save w/AnyPredicate");
    path_fail!(V::start(Nodes(vec!("D", "B"))).Save(Predicates(vec!("foo", "bar")), Tag("target")),
               "should fail to compile path.Save w/Predicates");
    path_fail!(V::start(Nodes(vec!("D", "B"))).Save(Predicate("follows"), AnyTag),
               "should fail to compile path.Save w/AnyTag");
    path_fail!(V::start(Nodes(vec!("D", "B"))).Save(Predicate("follows"), Tags(vec!("foo", "bar"))),
               "should fail to compile path.Save w/AnyTag"); */

    // == Joining ==

    // path.Intersect / path.And

    let mut cFollows = V::prepare(); cFollows.From(Node("C")).Out(Predicate("follows"), AnyTag);
    let mut dFollows = V::prepare(); dFollows.From(Node("D")).Out(Predicate("follows"), AnyTag);

    path_eq!(cFollows.clone().Intersect(&dFollows),
             "g.V(\"C\").Out(\"follows\").And(g.V(\"D\").Out(\"follows\"))");
    path_eq!(cFollows.clone().And(&dFollows),
             "g.V(\"C\").Out(\"follows\").And(g.V(\"D\").Out(\"follows\"))");

    // path.Union / path.Or

    let mut cFollows = V::start(Node("C")); cFollows.Out(Predicate("follows"), AnyTag);
    let mut dFollows = V::start(Node("D")); dFollows.Out(Predicate("follows"), AnyTag);

    path_eq!(cFollows.clone().Union(&dFollows),
             "g.V(\"C\").Out(\"follows\").Or(g.V(\"D\").Out(\"follows\"))");
    path_eq!(cFollows.clone().Or(&dFollows),
             "g.V(\"C\").Out(\"follows\").Or(g.V(\"D\").Out(\"follows\"))");

    // == Morphisms ==

    // path.Follow

    let mut friendOfFriend = M::start("friendOfFriend");
            friendOfFriend.Out(Predicate("follows"), AnyTag)
                          .Out(Predicate("follows"), AnyTag);
    path_eq!(friendOfFriend, "g.M().Out(\"follows\").Out(\"follows\")");

    path_eq!(V::start(Node("C")).Follow(&friendOfFriend).Has(Predicate("status"), Node("cool_person")),
             "var friendOfFriend = g.M().Out(\"follows\").Out(\"follows\");g.V(\"C\").Follow(friendOfFriend).Has(\"status\",\"cool_person\")");

    // path.FollowR

    path_eq!(V::start(AnyNode).Has(Predicate("status"), Node("cool_person")).FollowR(&friendOfFriend),
             "var friendOfFriend = g.M().Out(\"follows\").Out(\"follows\");g.V().Has(\"status\",\"cool_person\").FollowR(friendOfFriend)");

    // == Query finals ==

    path_eq!(V::start(AnyNode).Out(Predicate("follows"), AnyTag).All(),
             "g.V().Out(\"follows\").All()");

    path_eq!(V::start(Node("foo")).Out(Predicate("follows"), AnyTag).GetLimit(5),
             "g.V(\"foo\").Out(\"follows\").GetLimit(5)");

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

}
