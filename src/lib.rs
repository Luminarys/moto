#[macro_use]
extern crate moto_derive;

pub trait Store {
    type A;

    fn dispatch(self, a: Self::A) -> Self;
}

pub trait SubStore {
    type A;

    fn dispatch(self, a: Self::A) -> Self;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    enum Action {
        Inc,
        Dec,
        Append(String),
    }

    #[derive(Debug, Default, Store)]
    #[moto(action = "Action")]
    #[moto(middleware = "foo, logger")]
    struct State {
        #[moto(reducers = "counter")]
        counter: i64,
        #[moto(reducers = "appender")]
        appender: String,
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
    
    fn foo<F: Fn(State, Action) -> State>(store: State, next: F, action: Action) -> State {
        println!("Foo!");
        next(store, action)
    }
    
    fn logger<F: Fn(State, Action) -> State>(store: State, next: F, action: Action) -> State {
        println!("Dispatching {:?}", action);
        let result = next(store, action);
        println!("Next state {:?}", result);
        result
    }


    #[test]
    fn it_works() {
        let s = State {
            counter: 0,
            appender: "".to_owned(),
        };
        let s = s.dispatch(Action::Inc);
        assert_eq!(s.counter, 1);
        let s = s.dispatch(Action::Dec);
        assert_eq!(s.counter, 0);
        let s = s.dispatch(Action::Append("hi".to_owned()));
        assert_eq!(s.appender, "hi".to_owned());
    }
}
