#[macro_use]
extern crate moto_derive;
extern crate moto;

fn main() {
    let mut s = moto::Store::new::<EmptyMW>(Todos {
        todos: vec![],
        visibility: Visibility::All,
    });

    s.subscribe(Printer);

    s.dispatch(Action::Add(Todo {
        id: 0,
        text: "First thing!".to_owned(),
        completed: false,
    }));

    s.dispatch(Action::Add(Todo {
        id: 1,
        text: "Second thing!".to_owned(),
        completed: false,
    }));

    s.dispatch(Action::Toggle(0));

    s.dispatch(Action::SetVisibility(Visibility::Completed));

    s.dispatch(Action::SetVisibility(Visibility::Active));
}

#[derive(Debug, Clone)]
struct Todo {
    id: usize,
    text: String,
    completed: bool,
}

enum Action {
    Add(Todo),
    SetVisibility(Visibility),
    Toggle(usize),
}

#[derive(Clone, Copy, PartialEq)]
enum Visibility {
    All,
    Active,
    Completed,
}

#[derive(Reducer)]
struct Todos {
    #[moto(reducers = "add_todo, toggle_todo")]
    todos: Vec<Todo>,
    #[moto(reducers = "set_visibility")]
    visibility: Visibility,
}

#[derive(Middleware)]
pub struct EmptyMW;

struct Printer;

impl moto::Subscriber<Todos> for Printer {
    fn update(&mut self, store: &mut moto::Store<Todos>) {
        println!("Current TODOS:");
        let filter = store.get_state().visibility;
        let todos = store.get_state().todos.iter().filter(|t| match filter {
            Visibility::All => true,
            Visibility::Active => !t.completed,
            Visibility::Completed => t.completed,
        });
        for todo in todos {
            println!("TODO: {:?}", todo);
        }
        println!("");
    }
}

fn add_todo(mut todos: Vec<Todo>, action: &Action) -> moto::Result<Vec<Todo>> {
    match action {
        &Action::Add(ref t) => {
            todos.push(t.clone());
            Err(todos)
        }
        _ => Ok(todos),
    }
}

fn toggle_todo(mut todos: Vec<Todo>, action: &Action) -> moto::Result<Vec<Todo>> {
    match action {
        &Action::Toggle(id) => {
            let res = todos.iter_mut().find(|t| t.id == id).map(|t| {
                t.completed = !t.completed
            });
            if res.is_some() { Err(todos) } else { Ok(todos) }
        }
        _ => Ok(todos),
    }
}

fn set_visibility(visibility: Visibility, action: &Action) -> moto::Result<Visibility> {
    match action {
        &Action::SetVisibility(ref v) if v != &visibility => Err(*v),
        _ => Ok(visibility),
    }
}
