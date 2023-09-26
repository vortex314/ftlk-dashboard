#![allow(dead_code)]
#![allow(unused_imports)]
use std::cell::RefCell;
use std::rc::Rc;

//use tokio::prelude::*;


struct Source {

}
impl Source {
    async fn next(&self) -> String {
        "hello ".to_string()
    }
}
struct Pipe {
    
}
struct Sink<'a> {
    source : Option<&'a mut Source>
}

impl Sink<'_> {
    fn new() -> Sink<'static> {
        Sink { source : None }
    }
    fn add_source(&mut self,source:&mut Source) {
        self.source = Some(source);
    }
    async fn next(&mut self) -> String {
        self.source.as_mut().unwrap().next().await 
    }
}



#[tokio::main]
async fn main() -> () {
    let mut _source =  Source {};
    let mut _sink = Sink::new();
    _source.next().await;
    _sink.add_source(&mut _source);
    _sink.next().await;

    println!("Hello world");


}