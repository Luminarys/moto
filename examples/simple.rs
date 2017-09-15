extern crate moto;
#[macro_use]
extern crate moto_derive;

use std::fmt::Debug;

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

fn main() {
    let mut s = moto::Store::new::<MW>(Thing {
        counter: 0,
        appender: "".to_owned(),
        sub_state: SubThing { toggle: false },
    });

    s.subscribe(Sub);

    s.dispatch(Action::Inc);
    println!("Counter is now: {:#?}", s.get_state().counter);
    s.dispatch(Action::Dec);
    println!("Counter is now: {:#?}", s.get_state().counter);
    s.dispatch(Action::Append("foo".to_owned()));
    println!("Appender is now: {:#?}", s.get_state().appender);
}

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
fn logger<R, F, A>(store: &mut moto::Store<R>, next: F, action: A)
where
    R: moto::Reducer<Action = A> + Debug,
    F: Fn(&mut moto::Store<R>, A),
    A: Debug,
{
    println!("dispatching {:?}", action);
    next(store, action);
    println!("next state {:#?}", store.get_state());
}

struct Sub;

impl moto::Subscriber<Thing> for Sub {
    fn update(&mut self, _: &mut moto::Store<Thing>) {
        println!("changed!");
    }
}
