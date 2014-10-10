extern crate cayley;

use cayley::graph::{Graph, V1};
use cayley::graph::{GraphNodes, GraphNode};
use cayley::path::{Vertex, Query, Path};
use cayley::selector::AnyNode;

#[test]
fn main() {

    // echo "graph.Vertex('Humphrey Bogart').All()" | http --verbose POST localhost:64210/api/v1/query/gremlin Content-Type:text/plain

    // cayley::make_and_print_request("http://localhost:64210/api/v1/query/gremlin", "graph.Vertex(\"Humphrey Bogart\").All()");

    match Graph::new("localhost", 64210, V1) {

        Err(error) => fail!(error),
        Ok(graph) => {

            /* TODO: test saving Morphism */

            match graph.find(Vertex::start(AnyNode).All()) {

                Err(error) => fail!(error.to_string()),
                Ok(GraphNodes(nodes)) => {
                    assert!(nodes.len() > 0);
                }

            };

            match graph.find(Vertex::start(AnyNode).GetLimit(5)) {

                Err(error) => fail!(error.to_string()),
                Ok(GraphNodes(nodes)) => {
                    assert_eq!(nodes.len(), 5);
                }

            };

            /* match graph.vertex(Specific("Humphrey Bogart".to_string())).all() {

                Err(error) => fail!(error.to_string()),
                Ok(nodes) => {
                    assert_eq!(nodes.len(), 1);
                    //assert_eq!(nodes.iter().next().unwrap().id().as_slice(), "Humphrey Bogart");
                }

            }

            match graph.v(Specific("Humphrey Bogart".to_string()))
                       ._in("name")
                       .all() {

                Err(error) => fail!(error.to_string()),
                Ok(nodes) => {
                    assert_eq!(nodes.len(), 1);
                    //assert_eq!(nodes.iter().next().unwrap().id().as_slice(), ":/en/humphrey_bogart");
                }

            }

            match graph.v(Specific("Casablanca".to_string()))
                       ._in("name")
                       .all() {

                Err(error) => fail!(error.to_string()),
                Ok(nodes) => {
                    assert_eq!(nodes.len(), 1);
                    //assert_eq!(nodes.iter().next().unwrap().id().as_slice(), ":/en/casablanca_194");
                }

            } */

        }

    }

    //
    // a = graph.v().has("name", "Casablanca").all();
    // assert_eq!(a.len(), 1);
    // assert_eq!(a[0].id, ":/en/casablanca_194");
    //
    // a = graph.v()
    //          .has("name", "Casablanca")
    //          .out("/film/film/starring")
    //          .out("/film/performance/actor")
    //          .out("name")
    //          .all();
    // assert!(a.len() > 0);
    //
    // let film_to_actor = graph.morphism()
    //                          .out("/film/film/starring")
    //                          .out("/film/performance/actor");
    //
    // a = graph.v()
    //          .has("name", "Casablanca")
    //          .follow(film_to_actor)
    //          .out("name").all();
    // assert!(a.len() > 0);

    // it('test Emit', function(done) {
    //     this.timeout(10000);
    //     g.V("Casablanca").ForEach(function(d) { g.Emit(d); }, function(err, result) {
    //         assert(result.length);
    //         done(err);
    //     });
    // });
    //
    // it('test type shape', function(done) {
    //     this.timeout(10000);
    //     var graph = g.type('shape');
    //     graph.V("Casablanca").All(function(err, result) {
    //         assert(!result.links && result.nodes);
    //         done(err);
    //     });
    // });
    //
    // it('test write', function(done)  {
    //     client.write([{
    //         subject: "/zh/new_movie",
    //         predicate: "name",
    //         object: "New Movie"
    //     }], function(err) {
    //         if(err) return done(err);
    //         g.V('New Movie').All(function(err, result) {
    //             if(err) return done(err);
    //             assert.equal(result.length, 1);
    //             assert.equal(result[0].id, "New Movie");
    //             client.delete([{
    //                 subject: "/zh/new_movie",
    //                 predicate: "name",
    //                 object: "New Movie"
    //             }], function(err) {
    //                 if(err) return done(err);
    //                 g.V('New Movie').All(function(err, result) {
    //                     assert.equal(result.length, 1);
    //                     assert.equal(result[0].id, "");
    //                     done(err);
    //                 });
    //             });
    //         });
    //     });
    // });

}
