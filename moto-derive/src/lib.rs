extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;

#[proc_macro_derive(Store, attributes(moto))]
pub fn store(input: TokenStream) -> TokenStream {
    let s = input.to_string();
    let ast = syn::parse_derive_input(&s).unwrap();
    let gen = impl_store(&ast);
    gen.parse().unwrap()
}

fn impl_store(ast: &syn::DeriveInput) -> quote::Tokens {
    let mut action_name_opt = None;
    let mut middleware_names = Vec::new();
    for a in &ast.attrs {
        if let syn::MetaItem::List(ref i, ref v) = a.value {
            if i == &syn::Ident::from("moto") {
                if let Some(&syn::NestedMetaItem::MetaItem(syn::MetaItem::NameValue(ref name, syn::Lit::Str(ref value, _)))) = v.get(0) {
                    if name == &syn::Ident::from("action") {
                        action_name_opt = Some(syn::Ident::from(value.clone()));
                    } else if name == &syn::Ident::from("middleware") {
                        for mw in value.split(",") {
                            middleware_names.push(mw.to_owned());
                        }
                    }
                }
            }
        }
    }

    let store_name = &ast.ident;
    let action_name = action_name_opt.expect("Store must have attribute for action");

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
                                reducers.push((field.ident.as_ref().unwrap().clone(), reducer));
                            }
                        }
                    }
                }
            }
        }
        reducers
    });
    reducers.reverse();

    let reducer_ast = reducers.into_iter().fold(quote! { let mut changed = false; }, |prev_ast, (field, reducer)| {
        let reducer_name = syn::Ident::from(reducer);
        quote! {
            #prev_ast
            store.#field = match #reducer_name(store.#field, &action) {
                Ok(t) => t,
                Err(t) => { changed = true; t }
            };
        }
    });
    let base_dispatch = quote! {
        fn dispatch(mut store: #store_name, action: #action_name) -> #store_name {
            #reducer_ast
            println!("Changed: {}", changed);
            store
        }
    };

    middleware_names.reverse();
    let (middleware_name, middleware) = middleware_names
        .into_iter()
        .fold((syn::Ident::from("dispatch"), base_dispatch), |(prev_name, prev), name| {
        let iname = syn::Ident::from(name.clone());
        let func_name = syn::Ident::from(name.clone() + "_dispatch");
        (func_name.clone(), quote! {
            fn #func_name(store: #store_name, a: #action_name) -> #store_name {
                #prev
                #iname(store, #prev_name, a)
            }
        })
    });

    quote! {
        impl Store for #store_name {
            type A = #action_name;

            fn dispatch(self, a: Self::A) -> Self {
                #middleware

                #middleware_name(self, a)
            }
        }
    }
}
