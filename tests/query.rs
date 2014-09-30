use cayley::{Vertex};
use cayley::{EveryNode, Node, Nodes};

#[test]
fn main() {

    assert_eq!(Vertex::start(EveryNode).compile(), None)

    assert_eq!(Vertex::start(EveryNode).all().compile(),
              Some("g.Vertex().all()"))

    assert_eq!(Vertex::start(Node("Foo")).all().compile(),
              Some("g.Vertex(\"Foo\").all()"))
}
