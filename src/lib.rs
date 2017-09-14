#[macro_use]
extern crate moto_derive;

use std::fmt::Debug;
use std::mem;

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

pub trait Reducer: Debug {
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

    #[derive(Debug)]
    enum Action {
        Inc,
        Dec,
        Append(String),
        Nothing,
    }

    #[derive(Debug, Reducer)]
    struct Thing {
        #[moto(reducers = "counter")]
        counter: i64,
        #[moto(reducers = "appender")]
        appender: String,
        #[moto(sub_reducer)]
        sub_state: SubThing,
    }

    #[derive(Debug, Reducer)]
    struct SubThing {
        #[moto(reducers = "toggle")]
        toggle: bool,
    }

    #[derive(Middleware)]
    #[moto(middleware = "logger")]
    #[moto(reducer_bounds = "Debug")]
    #[moto(action_bounds = "Debug")]
    struct MW;

    fn counter(state: i64, a: &Action) -> Result<i64, i64> {
        match a {
            &Action::Inc => Err(state + 1),
            &Action::Dec => Err(state - 1),
            _ => Ok(state),
        }
    }

    fn appender(state: String, a: &Action) -> Result<String, String> {
        match a {
            &Action::Append(ref s) => Err(state + s),
            _ => Ok(state),
        }
    }

    fn toggle(state: bool, a: &Action) -> Result<bool, bool> {
        match a {
            &Action::Nothing => Ok(state),
            _ => Err(!state),
        }
    }

    /// A more complex generic middleware - works over any Store/Action that implement Debug
    fn logger<R, F, A>(store: &mut Store<R>, next: F, action: A)
    where
        R: Reducer<Action = A> + Debug,
        F: Fn(&mut Store<R>, A),
        A: Debug,
    {
        println!("Dispatching {:?}", action);
        next(store, action);
        println!("Next state {:?}", store.get_state());
    }

    struct Sub;

    impl Subscriber<Thing> for Sub {
        fn update(&mut self, _: &mut Store<Thing>) {
            println!("Changed!");
        }
    }

    #[test]
    fn it_works() {
        let mut s = Store::new::<MW>(Thing {
            counter: 0,
            appender: "".to_owned(),
            sub_state: SubThing { toggle: false },
        });
        s.subscribe(Sub);

        s.dispatch(Action::Inc);
        assert_eq!(s.get_state().counter, 1);
        assert_eq!(s.get_state().sub_state.toggle, true);
        s.dispatch(Action::Dec);
        assert_eq!(s.get_state().counter, 0);
        assert_eq!(s.get_state().sub_state.toggle, false);
        s.dispatch(Action::Dec);
        s.dispatch(Action::Append("hi".to_owned()));
        assert_eq!(s.get_state().appender, "hi".to_owned());
        s.dispatch(Action::Nothing);

        panic!(0);
    }
}
