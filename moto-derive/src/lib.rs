extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;

#[proc_macro_derive(Reducer, attributes(moto))]
pub fn reducer(input: TokenStream) -> TokenStream {
    let s = input.to_string();
    let ast = syn::parse_derive_input(&s).unwrap();
    let gen = impl_reducer(&ast);
    gen.parse().unwrap()
}

#[proc_macro_derive(Middleware, attributes(moto))]
pub fn middleware(input: TokenStream) -> TokenStream {
    let s = input.to_string();
    let ast = syn::parse_derive_input(&s).unwrap();
    let gen = impl_middleware(&ast);
    gen.parse().unwrap()
}

enum Field<'a> {
    Value(&'a str),
    SubReducer,
}

fn impl_reducer(ast: &syn::DeriveInput) -> quote::Tokens {
    let store_name = &ast.ident;

    let fields = match &ast.body {
        &syn::Body::Enum(_) => panic!("Enums not supported!"),
        &syn::Body::Struct(syn::VariantData::Struct(ref fields)) => fields,
        _ => panic!("Only structs with named fields are supported!"),
    };
    let mut reducers = fields.iter().fold(vec![], |mut reducers, field| {
        for a in &field.attrs {
            if let syn::MetaItem::List(ref i, ref v) = a.value {
                if i == &syn::Ident::from("moto") {
                    if let Some(&syn::NestedMetaItem::MetaItem(syn::MetaItem::NameValue(ref name, syn::Lit::Str(ref value, _)))) = v.get(0) {
                        if name == &syn::Ident::from("reducers") {
                            for reducer in value.split(",") {
                                let f = Field::Value(reducer);
                                reducers.push((field.ident.as_ref().unwrap().clone(), f));
                            }
                        }
                    } else if let Some(&syn::NestedMetaItem::MetaItem(syn::MetaItem::Word(ref name))) = v.get(0) {
                        if name == &syn::Ident::from("sub_reducer") {
                            let f = Field::SubReducer;
                            reducers.push((field.ident.as_ref().unwrap().clone(), f));
                        }
                    }
                }
            }
        }
        reducers
    });
    reducers.reverse();

    let base_ast = quote! {
        let mut changed = false;
        // From take_mut
        fn take<T, F>(mut_ref: &mut T, closure: F) where F: FnOnce(T) -> T {
            use std::ptr;
            use std::panic;

            unsafe {
                let old_t = ptr::read(mut_ref);
                let new_t = panic::catch_unwind(panic::AssertUnwindSafe(|| closure(old_t)))
                    .unwrap_or_else(|_| ::std::process::abort());
                ptr::write(mut_ref, new_t);
            }
        }
    };
    let reducer_ast = reducers.into_iter().fold(base_ast, |prev_ast, (field, reducer)| {
        match reducer {
            Field::Value(v) => {
                let reducer_name = syn::Ident::from(v);
                quote! {
                    #prev_ast
                    take(&mut self.#field, |f| {
                        match #reducer_name(f, action) {
                            Ok(t) => t,
                            Err(t) => { changed = true; t }
                        }
                    });
                }
            }
            Field::SubReducer => {
                quote! {
                    #prev_ast
                    changed |= self.#field.dispatch(action);
                }
            }
        }
    });

    let dummy_const = syn::Ident::new(format!("_IMPL_REDUCER_FOR_{}", store_name));
    quote! {
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const #dummy_const: () = {
            extern crate moto as _moto;

            impl _moto::Reducer for #store_name {
                type Action = Action;

                fn dispatch(&mut self, action: &Self::Action) -> bool {
                    #reducer_ast
                    changed
                }
            }
        };
    }
}

fn impl_middleware(ast: &syn::DeriveInput) -> quote::Tokens {
    let mut middleware_names = Vec::new();
    let mut r_bound_names = Vec::new();
    let mut a_bound_names = Vec::new();

    for a in &ast.attrs {
        if let syn::MetaItem::List(ref i, ref v) = a.value {
            if i == &syn::Ident::from("moto") {
                if let Some(&syn::NestedMetaItem::MetaItem(syn::MetaItem::NameValue(ref name, syn::Lit::Str(ref value, _)))) = v.get(0) {
                    if name == &syn::Ident::from("middleware") {
                        for mw in value.split(",") {
                            middleware_names.push(mw.to_owned());
                        }
                    } else if name == &syn::Ident::from("reducer_bounds") {
                        for mw in value.split(",") {
                            r_bound_names.push(mw.to_owned());
                        }
                    } else if name == &syn::Ident::from("action_bounds") {
                        for mw in value.split(",") {
                            a_bound_names.push(mw.to_owned());
                        }
                    }
                }
            }
        }
    }

    let mw_name = &ast.ident;

    match &ast.body {
        &syn::Body::Enum(_) => panic!("Enums not supported!"),
        _ => { }
    }

    let base_dispatch = quote! {
        fn dispatch<R: _moto::Reducer>(s: &mut _moto::Store<R>, a: R::Action) {
            s.reduce(a);
        }
    };

    let bf = |prev, bound| {
        let bn = syn::Ident::from(bound);
        quote! {
            #prev #bn +
        }
    };
    let r_bounds = r_bound_names.into_iter().fold(quote! {
        R: _moto::Reducer<Action = A> +
    }, &bf);

    let a_bounds = a_bound_names.into_iter().fold(quote! {
        A:
    }, &bf);

    middleware_names.reverse();
    let (middleware_name, middleware) = middleware_names
        .into_iter()
        .fold((syn::Ident::from("dispatch"), base_dispatch), |(prev_name, prev), name| {
        let iname = syn::Ident::from(name.clone());
        let func_name = syn::Ident::from(name.clone() + "_dispatch");
        (func_name.clone(), quote! {
            fn #func_name<R, A>(store: &mut _moto::Store<R>, a: R::Action) where #r_bounds, #a_bounds  {
                #prev
                #iname(store, #prev_name, a);
            }
        })
    });

    let dummy_const = syn::Ident::new(format!("_IMPL_MIDDLEWARE_FOR_{}", middleware_name));

    quote! {
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const #dummy_const: () = {
            extern crate moto as _moto;

            impl<R, A> _moto::Middleware<R> for #mw_name where #r_bounds, #a_bounds {
                fn apply(store: &mut _moto::Store<R>, action: A) {

                    #middleware

                    #middleware_name(store, action);
                }
            }
        };
    }
}
