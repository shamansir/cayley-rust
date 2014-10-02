#![feature(macro_rules)]

extern crate cayley;

use cayley::Vertex as V;
use cayley::Morphism as M;
use cayley::{EveryNode, Node, Nodes};

#[test]
fn main() {

    // Examples from: https://github.com/google/cayley/blob/master/docs/GremlinAPI.md

    macro_rules! path_eq(
        ($src:expr, $res:expr) => (
            assert_eq!($src.compile(), $res);
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

    // == Vertex ==

    // can be compiled, but not executed
    path_eq!(V::start(AnyNode), "g.V()");

    // can be compiled, but not executed
    path_eq!(V::start(Node("foo")), "g.V(\"foo\")");

    // can be compiled, but not executed
    path_eq!(V::start(Nodes(vec!("foo", "bar"))), "g.V(\"foo\", \"bar\")");

    path_eq!(V::start(AnyNode).All(), Some("g.V().all()"));

    path_eq!(V::start(Node("foo")).All(),
             Some("g.V(\"foo\").all()"));

    path_eq!(V::start(Nodes(vec!("foo", "bar"))).All(),
             Some("g.V(\"foo\", \"bar\").all()"));

    path_eq!(V::start(Nodes(vec!("foo", "bar"))).All(),
             Some("g.V(\"foo\", \"bar\").all()"));

    // == Morphism ==

    path_eq!(M::start().Out(Predicate("foo"), AnyTag)
                       .Out(Predicate("bar"), AnyTag),
             Some("g.M().Out(\"foo\").Out(\"bar\")"));

    path_eq!(M::start().Out(Predicate("foo"), Tags(vec!("tag1", "tag2")))
                       .Out(Predicate("bar"), Tag("tag0")),
             Some("g.M().Out(\"foo\", [\"tag1\", \"tag2\"]).Out(\"bar\", \"tag0\")"));

    // == Emit ==

    /* TODO: */

    // == Basic Traversals ==

    // path.Out

    path_eq!(V::start(Node("C")).Out(Predicate("follows"), AnyTag),
             Some("g.V(\"C\").Out(\"follows\")"));

    path_eq!(V::start(Node("A")).Out(Predicate("follows"), AnyTag)
                                     .Out(Predicate("follows"), AnyTag),
             Some("g.V(\"A\").Out(\"follows\").Out(\"follows\")"));

    path_eq!(V::start(Node("D")).Out(AnyPredicate, AnyTag),
             Some("g.V(\"D\").Out()"));

    path_eq!(V::start(Node("D")).Out(Predicates(vec!("follows", "status")), AnyTag),
             Some("g.V(\"D\").Out([\"follows\", \"status\")]"));

    path_eq!(V::start(Node("D")).Out(Query(V::start(Node("status"))), Tag("pred")),
             Some("g.V(\"D\").Out(g.V(\"status\"), \"pred\")"));

    // path.In

    path_eq!(V::start(Node("cool_person")).In(Predicate("status")),
             Some("g.V(\"cool_person\").In(\"status\")"));

    path_eq!(V::start(Node("B")).In(Predicate("follows")),
             Some("g.V(\"B\").In(\"follows\")"));

    path_eq!(V::start(Node("E")).Out(Predicate("follows")).In(Predicate("follows")),
             Some("g.V(\"B\").In(\"follows\").Out(\"follows\")"));

    path_eq!(V::start(Node("E")).Out(Predicate("follows")).In(Predicate("follows")),
             Some("g.V(\"B\").In(\"follows\").Out(\"follows\")"));

    // path.Both

    path_eq!(V::start(Node("F")).Both(Predicate("follows")),
             Some("g.V(\"F\").Both(\"follows\")"));

    // path.Is

    path_eq!(V::start(AnyNode).Out(Predicate("follows")).Is(Node("B")),
             Some("g.V().Out(\"follows\").Is(\"B\")"));

    path_eq!(V::start(AnyNode).Out(Predicate("follows")).Is(Nodes(vec!("B", "C"))),
             Some("g.V().Out(\"follows\").Is(\"B\", \"C\")"));

    // path.Has

    path_eq!(V::start(AnyNode).Has(Predicate("follows"), Node("B")),
             Some("g.V().Has(\"follows\", \"B\")"));

    // == Tagging ==

    // path.Tag / path.As

    path_eq!(V::start(AnyNode).As(Tag("start")).Out(Predicate("status"))),
             Some("g.V().As(\"start\").Out(\"status\")"));

    path_eq!(V::start(AnyNode).Tag(Tags(vec!("foo", "bar"))).Out(Predicate("status"))),
             Some("g.V().Tag(\"foo\", \"bar\").Out(\"status\")"));

    // path.Back

    path_eq!(V::start(AnyNode).As(Tag("start")).Out(Predicate("status")).
                              .Back(Tag("start").In(Predicate("follows")))),
             Some("g.V().As(\"start\").Out(\"status\").Back(\"start\").In(\"follows\")"));

    // path.Save

    path_eq!(V::start(Nodes(vec!("D", "B"))).Save(Predicate("follows"), Tag("target")),
             Some("g.V(\"D\", \"B\").Save(\"follows\", \"target\")"));

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

    let cFollows = V::start(Node("C")).Out("follows");
    let dFollows = V::start(Node("D")).Out("follows");

    path_eq!(cFollows.clone().Intersect(Query(dFollows)),
             Some("g.V(\"C\").Out(\"follows\").And(g.V(\"D\").Out(\"follows\"))"));
    path_eq!(cFollows.clone().And(Query(dFollows)),
             Some("g.V(\"C\").Out(\"follows\").And(g.V(\"D\").Out(\"follows\"))"));

    // path.Union / path.Or

    let cFollows = V::start(Node("C")).Out("follows");
    let dFollows = V::start(Node("D")).Out("follows");

    path_eq!(cFollows.clone().Union(Query(dFollows)),
             Some("g.V(\"C\").Out(\"follows\").Or(g.V(\"D\").Out(\"follows\"))"));
    path_eq!(cFollows.clone().Or(Query(dFollows)),
             Some("g.V(\"C\").Out(\"follows\").Or(g.V(\"D\").Out(\"follows\"))"));

    // == Morphisms ==

    // path.Follow

    let friendOfFriend = M::start("friendOfFriend").Out(Tag("follows")).Out(Tag("follows"));

    path_eq!(V::start(Node("C")).Follow(friendOfFriend).Has(Predicate("status"), Tag("cool_person")),
             Some("g.V(\"C\").Follow(friendOfFriend).Has(\"status\", \"cool_person\")"));

    // path.FollowR

    path_eq!(V::start(AnyNode)..Has(Predicate("status"), Tag("cool_person")).FollowR(friendOfFriend),
             Some("g.V(\"C\").Has(\"status\", \"cool_person\").FollowR(friendOfFriend)"));

    // == Query finals ==

    /* TODO: query.All() */

    /* TODO: query.GetLimit(5) */

    /* TODO: query.ToArray() */

    /* TODO: query.ToValue() */

    /* TODO: query.TagValue() */

    /* TODO: query.ForEach(callback), query.ForEach(limit, callback) */
}
