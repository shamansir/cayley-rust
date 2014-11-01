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

1. `RequestWriter` instance from [rust-http][] is created for every new query performed.
I planned to re-use same `RequestWriter` instance for every request since URL
is actually does not change, but Rust compiler appeared not to be happy enough with
this idea, since anyway `RequestWriter` should be mutable, by
spec, for `POST` requests, and so it just can't be passed here and there easily.

    Compiler is right, though, so we will wait for [teepee][] to be released,
to replace [rust-http][], may be there we will find some friendlier ways to do `POST`ing.

2. Currently `GraphNodes` returned from a query are stored as `HashMap<String, String>`,
which is a possible high memory over-use, since in most cases keys and values are immutable.
Though `json::Decoder` makes it really hard to use immutable types of strings there.
Anyway, I'll try to investigate in it and fix it among the first issues.

# TODO

## Features

Things from [Gremlin API][] still not implemented:

* `query.ToArray()`
* `query.ToValue()`
* `query.TagArray()`
* `query.TagValue()`
* `query.ForEach(callback), query.ForEach(limit, callback)` a.k.a `query.Map`
* `graph.Emit(data)`

## API improvements

* API change: Store `GraphNodes` as a map with immutable strings (see above, p.2 in
[Possible Drawbacks](#Possible-Drawbacks) section);
* API change: Implementing methods listed above will require to change a result of query
execution to some enum-wrapper, like `NodesMap(...)`, `NodeArray(...)`, `...`;
* Check if `Morphism` instance is already saved in this graph and fire an error, if it does;
* Some queries may produce additional errors while they just skip them, we need to store
an error inside a query and fire it when query is completed:
    * `.And`, `.Or`, `.Union`, `.Intersect` queries when the passed query is finalized;
    * `.And`, `.Or`, `.Union`, `.Intersect` queries when the passed queries failed to compile;
    * `Morphism` instance passed to `.Follow`/`FollowR` may not be saved when used;
    * Finalizers like `.All`, `.GetLimit`, ... may be called several times which should not happen;
* Maybe `Morphism` needs improvements, it's looks not so obvious in usage;
* Preparing `Vertex` for re-use requires to start a query using `.From()` selector,
and this case should be checked by API for sure;
* May be, better [Error API](http://www.hydrocodedesign.com/2014/05/28/practicality-with-rust-error-handling/);
* Some Path traits are public while they have no practical usage for user, like `Reuse`;
* [Log](http://doc.rust-lang.org/log/) executed queries;

* API change: Rather a thought to think on: This `mutable self` passed everywhere
may be solved with being a bit more functional and stopping using method chains — and using tuples
or vectors of enum-values instead, then iterating and mapping over them;
So, i.e. `Path` may appear as `enum` and be passed as a vector of values:
`( Has(Predicate("foo")), Tag("bar")), And((..., ..., ...)), Out(...) ), ( All, )`;
On the other hand, this way looks not so easy to read the whole thing as an actual chain of operations,
in comparison to method chains; Linked lists, then? Or operator overloading? Or macros?

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
