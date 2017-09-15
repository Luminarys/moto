#[macro_use]
extern crate moto_derive;

use std::mem;
use std::result;

pub type Result<T> = result::Result<T, T>;

pub struct Store<R: Reducer> {
    root: R,
    middleware: fn(&mut Store<R>, R::Action),
    subs: Vec<Box<Subscriber<R>>>,
}

pub trait Middleware<R: Reducer> {
    fn apply(&mut Store<R>, R::Action);
}

pub trait Subscriber<R: Reducer> {
    fn update(&mut self, &mut Store<R>);
}

pub trait Reducer {
    type Action;

    fn dispatch(&mut self, &Self::Action) -> bool;
}

impl<R: Reducer> Store<R> {
    pub fn new<M: Middleware<R>>(root: R) -> Store<R> {
        Store {
            root,
            middleware: M::apply,
            subs: vec![],
        }
    }

    pub fn dispatch(&mut self, action: R::Action) {
        (self.middleware)(self, action);
    }

    pub fn reduce(&mut self, action: R::Action) {
        if self.root.dispatch(&action) {
            let mut subs = mem::replace(&mut self.subs, Vec::with_capacity(0));
            for sub in &mut subs {
                sub.update(self);
            }
            self.subs = subs;
        }
    }

    pub fn get_state(&self) -> &R {
        &self.root
    }

    pub fn subscribe<S: Subscriber<R> + 'static>(&mut self, sub: S) -> usize {
        self.subs.push(Box::new(sub));
        0
    }

    pub fn unsubscribe(&mut self, token: usize) {
        // later, with a hashmap
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(true);
    }
}
