#![crate_name = "cayley"]

extern crate debug;
extern crate http;
extern crate url;
use http::client::RequestWriter;
use http::method::Get;
use http::headers::HeaderEnum;
use std::str;
use std::io::println;
use url::Url;

pub fn make_and_print_request(url: &str) {
    // echo "graph.Vertex('Humphrey Bogart').All()" |
    // http --verbose POST localhost:64210/api/v1/query/gremlin Content-Type:text/plain

    let url = Url::parse(url).ok().expect("Invalid URL :-(");
    let request: RequestWriter = RequestWriter::new(Get, url).unwrap();

    println!("[33;1mRequest[0m");
    println!("[33;1m=======[0m");
    println!("");
    println!("[1mURL:[0m {}", request.url);
    println!("[1mRemote address:[0m {}", request.remote_addr);
    println!("[1mMethod:[0m {}", request.method);
    println!("[1mHeaders:[0m");
    for header in request.headers.iter() {
        println!(" - {}: {}", header.header_name(), header.header_value());
    }

    println!("");
    println!("[33;1mResponse[0m");
    println!("[33;1m========[0m");
    println!("");
    let mut response = match request.read_response() {
        Ok(response) => response,
        Err(_request) => fail!("This example can progress no further with no response :-("),
    };
    println!("[1mStatus:[0m {}", response.status);
    println!("[1mHeaders:[0m");
    for header in response.headers.iter() {
        println!(" - {}: {}", header.header_name(), header.header_value());
    }
    println!("[1mBody:[0m");
    let body = match response.read_to_end() {
        Ok(body) => body,
        Err(err) => fail!("Reading response failed: {}", err),
    };
    println(str::from_utf8(body.as_slice()).expect("Uh oh, response wasn't UTF-8"));
}
