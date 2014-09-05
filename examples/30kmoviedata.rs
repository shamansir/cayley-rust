extern crate cayley;

// https://github.com/villadora/cayley.js/blob/master/test/cayley.js

fn main() {
    let graph = cayley::Graph::new("localhost", 64210);

    graph.vertex().get_limit(5);

    graph.vertex("Humphrey Bogart").all();

    graph.v("Humphrey Bogart").all();

    graph.v("Humphrey Bogart").in("name").all();

    graph.v("Casablanca").in("name").all();

    graph.v().has("name", "Casablanca").all();

    graph.v()
         .has("name", "Casablanca")
         .out("/film/film/starring")
         .out("/film/performance/actor")
         .out("name")
         .all();

    let film_to_actor = graph.morphism()
                             .out("/film/film/starring")
                             .out("/film/performance/actor");

    graph.v()
         .has("name", "Casablanca")
         .follow(film_to_actor)
         .out("name").all();

}
