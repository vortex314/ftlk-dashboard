use std::ops::ShrAssign;

pub mod limero;
pub trait Sink<T> {
    fn on(&mut self, value: & T);
}

pub struct Source<T:Sized> {
    pub subscribers: Vec<Box<dyn Sink<T>>>,
}

impl<T:Sized> Source<T> {
    pub fn new() -> Source<T> {
        Source {
            subscribers: Vec::new(),
        }
    }

    fn subscribe(&mut self, sink: Box<dyn Sink<T>>) {
        self.subscribers.push(sink);
    }

    fn emit(&mut self, value: & T) {
        for subscriber in self.subscribers.iter_mut() {
            subscriber.on(value);
        }
    }
}

impl<T> ShrAssign<Box<dyn Sink<T>> > for Source<T> {
    fn shr_assign(&mut self, sink:  Box<dyn Sink<T>>) {
        self.subscribe(sink);
    }
}