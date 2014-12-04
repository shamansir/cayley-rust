[![Build Status](https://secure.travis-ci.org/shamansir/cayley-rust.png)](https://travis-ci.org/shamansir/cayley-rust)

**In progress, but may be used for a simple things**

**NB:** Built over nightly Rust, but not guaranteed to be up-to-date with the
very latest version, since it is was made for fun. While I use it, though, it will
appear rather fresh for some time, I suppose.

## About

Cayley is a new graph-driven Database engine from Google, read about it in
[their Github][cayley] or in different articles.

## API Docs

[API Docs](http://shamansir.github.io/cayley-rust/cayley/index.html) (`rustdoc`).

## Code example

To get a first impression on how it looks in action, see
[test with queries to 30K-Movie database][30kmoviedata-test] and [path compilation test][path-compile-test] sources.

## Usage

Add these lines to your `Cargo.toml`:

```toml
[dependencies.cayley]
git = "https://github.com/shamansir/cayley-rust"
doc = false
```

And then, in any file you want to use Cayley, add:

```rust
extern crate cayley;
```

Just don't forget to start Cayley DB itself before every session:

`$ ./cayley http --dbpath=<your-database>`

To connect to a graph at `localhost:64210`, use code like this:

```rust
use cayley::{Graph, V1};

let graph = match Graph::new("localhost", 64210, V1) {
    Err(error) => panic!(error),
    Ok(graph) => graph
};
```

A simple query pattern looks like this:

```rust
use cayley::{GraphNode, GraphNodes};

use cayley::path::{Vertex, Query}; // NB! `Query` is required.
use cayley::selector::AnyNode;

//               The query itself
match graph.find(Vertex::start(AnyNode).All()) {

    Err(error) => panic!(error.to_string()),
    Ok(GraphNodes(nodes)) => {
        assert!(nodes.len() > 0);
        match nodes.iter().next() {
            Some(&GraphNode(ref first_node)) => {
                // node is a HashMap<String, String>
                println!("{:s}", first_node["id".to_string()]);
            },
            None => panic!("first node was not found")
        }
    }

};
```

**NB**: `Query` trait is required to be imported to give you access to `.All()`
method of a `Vertex` instance. If you feel you don't like it, feel free to support
[my post][trait-use-requirement-discuss] in Rust language discussions.

Look for more complex requests just [below](#syntax-examples).

## Syntax examples

Due to Rust strict typing, it's hard to transfer free-minded JS-inspired query
syntax from Cayley as-is. Though I decided to leave upper-case style for Query/Path
methods, since i.e. no method can be named `in()` in Rust, because `in` is a keyword.

Here are some parallels:

##### 1.

Gremlin:

`graph.Vertex().All()`

cayley-rust:

```rust
graph.find(Vertex::start(AnyNode).All())
```

##### 2.

Gremlin:

`graph.Vertex("C").Out("follows").GetLimit(5)`

cayley-rust:

```rust
graph.find(Vertex::start(Node("C")).OutP(Predicate("follows")).GetLimit(5))
```

##### 3.

Gremlin:

`var friendOfFriend = Morphism().Out("follows").Out("follows")`

cayley-rust:

```rust
let friendOfFriend = Morphism::start("friendOfFriend")
                              .OutP(Predicate("follows"))
                              .OutP(Predicate("follows"));
```

##### 4.

Gremlin:

`g.V("C").Follow(friendOfFriend).Has("status","cool_person")`

cayley-rust:

```rust
...
use cayley::path::Vertex as V;

let g = Graph::new(...);
let mut friendOfFriend = Morphism::start("friendOfFriend");
        friendOfFriend.OutP...;
...
g.find(V::start(Node("C"))
         .Follow(&friendOfFriend)
         .Has(Predicate("status"), Node("cool_person")));
```

# Possible drawbacks

I have no actual sense if it is correct to use `String`/`to_string()` in some cases, but I really tried to avoid that as much as possible, and I plan to try
further, if it's not. Or, please, you PRs are welcome.

# TODO

## Features

Things from [Gremlin API][] still not implemented:

* `query.ToArray()` — not supported with HTTP requests
* `query.ToValue()` — not supported with HTTP requests
* `query.TagArray()` — not supported with HTTP requests
* `query.TagValue()` — not supported with HTTP requests
* `query.ForEach(callback), query.ForEach(limit, callback)` a.k.a `query.Map`
* `graph.Emit(data)`

## API improvements

* API change: Store `GraphNodes` as a map with immutable strings (see above, p.2 in
[Possible Drawbacks](#Possible-Drawbacks) section);
* Preparing `Vertex` for re-use requires to start a query using `.From()` selector,
and this case should be checked by API for sure;
* May be, better [Error API](http://www.hydrocodedesign.com/2014/05/28/practicality-with-rust-error-handling/);

# Thanks

Thanks to all [Rust IRC][] members for help.

[rust-http]: https://github.com/chris-morgan/rust-http
[teepee]: https://github.com/teepee/teepee
[cayley]: https://github.com/google/cayley/

[Gremlin API]: https://github.com/google/cayley/blob/master/docs/GremlinAPI.md
[Rust IRC]: http://chat.mibbit.com/?server=irc.mozilla.org&channel=%23rust

[connection-test]: https://github.com/shamansir/cayley-rust/blob/master/tests/connection.rs
[path-compile-test]: https://github.com/shamansir/cayley-rust/blob/master/tests/path_compile.rs
[30kmoviedata-test]: https://github.com/shamansir/cayley-rust/blob/master/tests/30kmoviedata.rs

[trait-use-requirement-discuss]: http://discuss.rust-lang.org/t/no-requirement-to-import-a-trait-for-using-an-implemented-public-method-from-it/579
