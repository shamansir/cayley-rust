extern crate cayley;

use cayley::{Graph, GraphAccess};
use cayley::{NodeName, AnyNode};

#[test]
fn main() {

    // echo "graph.Vertex('Humphrey Bogart').All()" | http --verbose POST localhost:64210/api/v1/query/gremlin Content-Type:text/plain

    match Graph::new(GraphAccess {
            host: "localhost",
            port: 64210,
            version: "v1"
        }) {

        Err(error) => fail!(error),
        Ok(graph) => {

            let mut a = graph.v(AnyNode).all();
            assert!(a.len() > 0);

        }

    }

    //
    // let mut a = graph.v().all();
    // assert!(a.len() > 0);
    //
    // a = graph.v().get_limit(5);
    // assert_eq!(a.len(), 5);
    //
    // a = graph.vertex("Humphrey Bogart").all();
    // assert_eq!(a.len(), 1);
    // assert_eq!(a[0].id, "Humphrey Bogart");
    //
    // a = graph.v("Humphrey Bogart").in("name").all();
    // assert_eq!(a.len(), 1);
    // assert_eq!(a[0].id, ":/en/humphrey_bogart");
    //
    // a = graph.v("Casablanca").in("name").all();
    // assert_eq!(a.len(), 1);
    // assert_eq!(a[0].id, ":/en/casablanca_194");
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
