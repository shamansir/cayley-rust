extern crate cayley;

use cayley::graph::{Graph, V1};
use cayley::graph::{GraphNodes, GraphNode};
use cayley::path::{Morphism, Vertex, Path, Query};
use cayley::selector::{AnyNode, Node};
use cayley::selector::AnyTag;
use cayley::selector::Predicate;

#[test]
fn main() {

    // echo "graph.Vertex('Humphrey Bogart').All()" | http --verbose POST localhost:64210/api/v1/query/gremlin Content-Type:text/plain

    // cayley::make_and_print_request("http://localhost:64210/api/v1/query/gremlin", "graph.Vertex(\"Humphrey Bogart\").All()");

    match Graph::new("localhost", 64210, V1) {

        Err(error) => panic!(error),
        Ok(graph) => {

            /* TODO: test saving Morphism */

            match graph.find(vertex!(AnyNode => All)) {

                Err(error) => panic!(error.to_string()),
                Ok(GraphNodes(nodes)) => {
                    assert!(nodes.len() > 0);
                }

            };

            match graph.find(vertex!(AnyNode => GetLimit(5))) {

                Err(error) => panic!(error.to_string()),
                Ok(GraphNodes(nodes)) => {
                    assert_eq!(nodes.len(), 5);
                }

            };

            match graph.find(vertex!(Node("Humphrey Bogart") => All)) {

                Err(error) => panic!(error.to_string()),
                Ok(GraphNodes(nodes)) => {
                    assert_eq!(nodes.len(), 1);
                    match nodes.iter().next() {
                        Some(&GraphNode(ref humphrey)) => {
                            assert_eq!(humphrey["id".to_string()].as_slice(), "Humphrey Bogart");
                        },
                        None => panic!("first node was not found")
                    }
                }

            }

            match graph.find(vertex!(Node("Humphrey Bogart")
                                     -> In(Predicate("name"), AnyTag)
                                     => All)) {

                Err(error) => panic!(error.to_string()),
                Ok(GraphNodes(nodes)) => {
                    assert_eq!(nodes.len(), 1);
                    match nodes.iter().next() {
                        Some(&GraphNode(ref humphrey)) => {
                            assert_eq!(humphrey["id".to_string()].as_slice(), "/en/humphrey_bogart");
                            // was: ":/en/humphrey_bogart"
                        },
                        None => panic!("first node was not found")
                    }
                }

            }

            match graph.find(vertex!(Node("Casablanca")
                                     -> InP(Predicate("name"))
                                     => All)) {

                Err(error) => panic!(error.to_string()),
                Ok(GraphNodes(nodes)) => {
                    assert_eq!(nodes.len(), 1);
                    match nodes.iter().next() {
                        Some(&GraphNode(ref casablanca)) => {
                            assert_eq!(casablanca["id".to_string()].as_slice(), "/en/casablanca_1942");
                            // was: ":/en/casablanca_1942"
                        },
                        None => panic!("first node was not found")
                    }
                }

            }

            match graph.find(vertex!(AnyNode
                                     -> Has(Predicate("name"), Node("Casablanca"))
                                     -> OutP(Predicate("/film/film/starring"))
                                     -> OutP(Predicate("/film/performance/actor"))
                                     -> OutP(Predicate("name"))
                                     => All)) {

                Err(error) => panic!(error.to_string()),
                Ok(GraphNodes(nodes)) => {
                    assert!(nodes.len() > 0);
                }
            }

            let mut film_to_actor = Morphism::start("fta");
                    film_to_actor.OutP(Predicate("/film/film/starring"))
                                 .OutP(Predicate("/film/performance/actor"));
            match graph.find(Vertex::start(AnyNode)
                                    .Has(Predicate("name"), Node("Casablanca"))
                                    .Follow(&mut film_to_actor)
                                    .OutP(Predicate("name"))
                                    .All()) {

                Err(error) => panic!(error.to_string()),
                Ok(GraphNodes(nodes)) => {
                    println!("{}",nodes.len());
                    assert!(nodes.len() > 0);
                }
            }

        }

    }

    // graph.write([ { subject: "Subject Node",
    //                 predicate: "Predicate Node",
    //                 object: "Object Node" }]);
    //
    // graph.delete([ { subject: "Subject Node",
    //                 predicate: "Predicate Node",
    //                 object: "Object Node" }]);

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
