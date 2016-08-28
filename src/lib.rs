extern crate rustc_serialize;
extern crate hyper;

use std::io::Read;
use std::collections::HashMap;
use rustc_serialize::{Encodable, json};

pub trait Transport {
  fn listen(&self);
  fn stop(&self);
  fn connect(&self);
  fn disconnect(&self);
  fn call<T>(&self, method: &str, data: &T) -> &str
    where T: Encodable;
}

pub struct Client<'a, T: 'a> {
  transports: &'a HashMap<&'a str, &'a T>
}

impl<'a, T: 'a + Transport> Client<'a, T> {
  fn new(transports: &'a HashMap<&'a str, &'a T>) -> Client<T> {
    Client {
      transports: &transports
    }
  }
  fn connect(&self) {
    for (service, t) in self.transports.iter() {
      t.connect()
    }
  }
  fn disconnect(&self) {
    for (service, t) in self.transports.iter() {
      t.disconnect()
    }
  }
  fn call<A>(&self, service: &str, method: &str, data: &A) -> &str 
    where A: Encodable {
    match self.transports.get(service) {
      Some(transport) =>  {
        let result = transport.call(method, data);
        result
      }
      _ => {
        println!("unknown service");
        let result = "error";
        result
      }
    }
  }
}

pub struct Service<'a, T: 'a, F: 'a> {
  transport: &'a T,
  methods: HashMap<&'a str, Box<F>>
}

impl<'a, T: 'a + Transport, F> Service<'a, T, F> {
  fn new(transport: &'a T) -> Service<T, F> {
    Service {
      transport: transport,
      methods: HashMap::new()
    }
  }
  fn listen(&self) {
    &self.transport.listen();
  }
  fn on(&mut self, method: &'a str, callback: Box<F>) 
    where F : Fn(String) -> String {
    self.methods.insert(&method, callback);
  }
}

fn post_json<T>(url: &str, payload: &T) -> hyper::Result<String>
    where T: Encodable {
    let client = hyper::Client::new();
    let body = json::encode(payload).unwrap();
    let mut response = try!(client.post(url).body(&body[..]).send());
    let mut buf = String::new();
    try!(response.read_to_string(&mut buf));
    Ok(buf)
}

pub struct HTTPTransport<'a> {
  host: &'a str,
  port: i16
}

impl<'a> HTTPTransport<'a> {
  fn new(host: &'a str, port: i16) -> HTTPTransport<'a> {
    HTTPTransport {
      host: host,
      port: port
    }
  }
}

impl<'a> Transport for HTTPTransport<'a> {
  fn listen(&self) {}
  fn stop(&self) {}
  fn connect(&self) {}
  fn disconnect(&self) {} 
  fn call<T>(&self, method: &str, data: &T) -> &str
    where T: Encodable {
    let url = format!("http://{}:{}/{}", self.host, self.port, "ht".to_owned());

    match post_json(&url, &data) {
      Ok(request) => request,
      Err(e) => {
        println!("error: {}", e);
        &String::from("error")
      }
    }

  }
}

#[derive(RustcEncodable)]
struct Request<'a> {
  hello: &'a str
}

fn go() {
  let tcp = HTTPTransport::new("127.0.0.1", 8080);
  let mut map = HashMap::new();
  map.insert("local", &tcp);
  let client = Client::new(&map);
  let req = Request {
    hello: "world"
  };
  println!("{}", client.call("local", "echo", &req));
}
