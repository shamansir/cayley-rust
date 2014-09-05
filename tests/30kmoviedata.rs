extern crate cayley;

#[test]
fn main() {
    let graph = cayley::Graph::new("localhost", 64210);

    let mut a = graph.vertex().all();
    assert!(a.len() > 0);

    a = graph.vertex().get_limit(5);
    assert_eq!(a.len(), 5);

    a = graph.vertex("Humphrey Bogart").all();
    assert_eq!(a.len(), 1);
    assert_eq!(a[0].id, "Humphrey Bogart");

    // graph.v("Humphrey Bogart").all();
    //
    // graph.v("Humphrey Bogart").in("name").all();
    //
    // graph.v("Casablanca").in("name").all();
    //
    // graph.v().has("name", "Casablanca").all();
    //
    // graph.v()
    //      .has("name", "Casablanca")
    //      .out("/film/film/starring")
    //      .out("/film/performance/actor")
    //      .out("name")
    //      .all();
    //
    // let film_to_actor = graph.morphism()
    //                          .out('/film/film/starring')
    //                          .out('/film/performance/actor');
    //
    // graph.v()
    //      .has("name", "Casablanca")
    //      .follow(film_to_actor)
    //      .out("name").all();

}
