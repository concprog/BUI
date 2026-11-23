//! `#[component]` for bui-charts: blenderui's own `#[component]` takes neither generics nor typed
//! children, and every chart is generic over its rows (`Dataset<T>`) and takes closures (`x={|r: &Row| r.t}`)
//! and series (`<TrendSeries .. />`) as children. This one writes the same props builder blenderui's
//! `rsx!` drives (`Tag::__builder()`, a setter per prop, `.children(..)`, `Complete`), so a chart is used
//! in markup like any blenderui component:
//!
//! ```ignore
//! #[component]
//! pub fn gauge<T: 'static, D: Dataset<T>, V: Fn(&T) -> f64 + 'static>(
//!     data: D,                 // required: a compile error names it when it is left out
//!     value: Option<V>,        // a callback: an unset one costs nothing
//!     #[into] color: Option<Hsla>,
//!     class: Option<SharedString>,
//! ) -> AnyElement { .. }
//! // rsx! { <Gauge data={rows} value={|r: &Row| r.cpu} class="h-56" /> }
//! ```
//!
//! A component that returns something other than an element (`-> TrendField<T>`) is a typed child: its
//! tag offers the value to the parent being built, whose `children: Children<TrendField<T>>` receives
//! them in markup order.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, format_ident, quote};
use syn::{FnArg, ItemFn, Pat, PatType, parse_macro_input, parse_quote};

#[proc_macro_attribute]
pub fn component(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    generate(input).unwrap_or_else(syn::Error::into_compile_error).into()
}

fn to_pascal(name: &str) -> String {
    name.split('_')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            chars.next().map_or_else(String::new, |first| first.to_uppercase().chain(chars).collect())
        })
        .collect()
}

fn option_inner(ty: &syn::Type) -> Option<&syn::Type> {
    let syn::Type::Path(path) = ty else { return None };
    let segment = path.path.segments.last()?;
    if segment.ident != "Option" {
        return None;
    }
    let syn::PathArguments::AngleBracketed(args) = &segment.arguments else { return None };
    match args.args.first()? {
        syn::GenericArgument::Type(inner) => Some(inner),
        _ => None,
    }
}

/// `Children` -> `AnyElement`, `Children<T>` -> `T`.
fn children_item(ty: &syn::Type) -> Option<syn::Type> {
    let syn::Type::Path(path) = ty else { return None };
    let segment = path.path.segments.last()?;
    if segment.ident != "Children" {
        return None;
    }
    match &segment.arguments {
        syn::PathArguments::None => Some(parse_quote!(::blenderui::element::AnyElement)),
        syn::PathArguments::AngleBracketed(args) => match args.args.first()? {
            syn::GenericArgument::Type(item) => Some(item.clone()),
            _ => None,
        },
        syn::PathArguments::Parenthesized(_) => None,
    }
}

fn is_element(ty: &syn::Type) -> bool {
    matches!(ty, syn::Type::Path(path) if path.path.segments.last().is_some_and(|s| s.ident == "AnyElement"))
}

/// A generic parameter bounded by `Fn*`: an optional prop of that type is a callback.
struct Callback {
    param: syn::Ident,
    bounds: Vec<syn::TypeParamBound>,
    /// the type of an unset callback: a `fn` pointer of the same signature
    unset: syn::Type,
}

fn callbacks(generics: &syn::Generics) -> Vec<Callback> {
    let mut found: Vec<(syn::Ident, Vec<syn::TypeParamBound>)> =
        generics.type_params().map(|param| (param.ident.clone(), param.bounds.iter().cloned().collect())).collect();
    for predicate in generics.where_clause.iter().flat_map(|clause| &clause.predicates) {
        if let syn::WherePredicate::Type(predicate) = predicate
            && let syn::Type::Path(path) = &predicate.bounded_ty
            && let Some(ident) = path.path.get_ident()
            && let Some((_, bounds)) = found.iter_mut().find(|(param, _)| param == ident)
        {
            bounds.extend(predicate.bounds.iter().cloned());
        }
    }
    found
        .into_iter()
        .filter_map(|(param, bounds)| {
            let unset = bounds.iter().find_map(|bound| {
                let syn::TypeParamBound::Trait(bound) = bound else { return None };
                let segment = bound.path.segments.last()?;
                let syn::PathArguments::Parenthesized(signature) = &segment.arguments else { return None };
                ["Fn", "FnMut", "FnOnce"].iter().any(|name| segment.ident == name).then(|| {
                    let (inputs, output) = (&signature.inputs, &signature.output);
                    let unset: syn::Type = parse_quote!(fn(#inputs) #output);
                    unset
                })
            })?;
            Some(Callback { param, bounds, unset })
        })
        .collect()
}

enum Role {
    /// set before the component renders; tracked in the builder's type
    Required,
    /// `Option<T>`: the setter takes a `T`
    Optional(syn::Type),
    /// `Option<F>`, `F: Fn(..)`: tracked in the builder's type, so an unset one needs no type
    Callback(usize),
    /// `children: Children` / `Children<T>`
    Children(syn::Type),
}

struct Prop {
    name: syn::Ident,
    ty: syn::Type,
    role: Role,
    into: bool,
}

/// A setter's parameter and how it stores it: strings convert, as do `#[into]` props.
fn setter_param(ty: &syn::Type, field: &syn::Ident, into: bool) -> (TokenStream2, TokenStream2) {
    let last = match ty {
        syn::Type::Path(path) => path.path.segments.last().map(|segment| segment.ident.to_string()),
        _ => None,
    };
    if into || matches!(last.as_deref(), Some("String" | "SharedString")) {
        return (quote!(impl ::std::convert::Into<#ty>), quote!(#field.into()));
    }
    if last.as_deref() == Some("AnyElement") {
        return (quote!(impl ::blenderui::element::IntoElement), quote!(::blenderui::element::IntoElement::into_any_element(#field)));
    }
    (ty.to_token_stream(), field.to_token_stream())
}

fn generic_arg(param: &syn::GenericParam) -> TokenStream2 {
    match param {
        syn::GenericParam::Type(param) => param.ident.to_token_stream(),
        syn::GenericParam::Lifetime(param) => param.lifetime.to_token_stream(),
        syn::GenericParam::Const(param) => param.ident.to_token_stream(),
    }
}

fn plain_param(param: &syn::GenericParam) -> TokenStream2 {
    match param {
        syn::GenericParam::Const(param) => {
            let (ident, ty) = (&param.ident, &param.ty);
            quote!(const #ident: #ty)
        }
        param => generic_arg(param),
    }
}

#[allow(clippy::too_many_lines)]
fn generate(mut input: ItemFn) -> syn::Result<TokenStream2> {
    let found = callbacks(&input.sig.generics);
    let mut props = Vec::new();
    for arg in &mut input.sig.inputs {
        let FnArg::Typed(PatType { pat, ty, attrs, .. }) = arg else {
            return Err(syn::Error::new_spanned(arg, "a component takes props, not self"));
        };
        let into = attrs.iter().any(|attr| attr.path().is_ident("into"));
        attrs.retain(|attr| !attr.path().is_ident("into"));
        let Pat::Ident(ident) = &**pat else {
            return Err(syn::Error::new_spanned(pat, "a prop is `name: Type`"));
        };
        let name = ident.ident.clone();
        if ["new", "render", "__builder"].iter().any(|reserved| name == reserved) {
            return Err(syn::Error::new(name.span(), format!("`{name}` is the props builder's; rename the prop")));
        }
        let role = match children_item(ty) {
            Some(item) if name == "children" => {
                **ty = parse_quote!(::std::vec::Vec<#item>);
                Role::Children(item)
            }
            _ => {
                let callback = option_inner(ty).and_then(|inner| match inner {
                    syn::Type::Path(path) => path.path.get_ident().and_then(|ident| found.iter().position(|c| c.param == *ident)),
                    _ => None,
                });
                match (callback, option_inner(ty)) {
                    (Some(index), _) => Role::Callback(index),
                    (None, Some(inner)) => Role::Optional(inner.clone()),
                    (None, None) => Role::Required,
                }
            }
        };
        props.push(Prop { name, ty: (**ty).clone(), role, into });
    }

    let tracked_params: Vec<&syn::Ident> = props
        .iter()
        .filter_map(|prop| match prop.role {
            Role::Callback(index) => Some(&found[index].param),
            _ => None,
        })
        .collect();
    let mut generics = input.sig.generics.clone();
    generics.params = std::mem::take(&mut generics.params)
        .into_iter()
        .filter(|param| !matches!(param, syn::GenericParam::Type(param) if tracked_params.contains(&&param.ident)))
        .collect();
    if let Some(clause) = &mut generics.where_clause {
        clause.predicates = std::mem::take(&mut clause.predicates)
            .into_iter()
            .filter(|predicate| {
                !matches!(predicate, syn::WherePredicate::Type(predicate)
                    if matches!(&predicate.bounded_ty, syn::Type::Path(path)
                        if path.path.get_ident().is_some_and(|ident| tracked_params.contains(&ident))))
            })
            .collect();
    }

    let returns_element = match &input.sig.output {
        syn::ReturnType::Default => true,
        syn::ReturnType::Type(_, ty) => is_element(ty),
    };
    let output: syn::Type = match &input.sig.output {
        syn::ReturnType::Type(_, ty) if !returns_element => (**ty).clone(),
        _ => parse_quote!(::blenderui::element::AnyElement),
    };
    if matches!(input.sig.output, syn::ReturnType::Default) {
        let block = &input.block;
        input.block = parse_quote!({ ::blenderui::element::IntoElement::into_any_element(#block) });
        input.sig.output = parse_quote!(-> ::blenderui::element::AnyElement);
    }

    let vis = &input.vis;
    let name = &input.sig.ident;
    let tag = format_ident!("{}", to_pascal(&name.to_string()), span = name.span());
    let props_ident = format_ident!("{}Props", tag, span = name.span());
    let params: Vec<&syn::GenericParam> = generics.params.iter().collect();
    let args: Vec<TokenStream2> = params.iter().map(|param| generic_arg(param)).collect();
    let plain: Vec<TokenStream2> = params.iter().map(|param| plain_param(param)).collect();
    let where_clause = &generics.where_clause;
    let where_predicates: Vec<&syn::WherePredicate> =
        generics.where_clause.as_ref().map(|clause| clause.predicates.iter().collect()).unwrap_or_default();
    let marker: Vec<TokenStream2> = params
        .iter()
        .filter_map(|param| match param {
            syn::GenericParam::Type(param) => Some(param.ident.to_token_stream()),
            syn::GenericParam::Lifetime(param) => {
                let lifetime = &param.lifetime;
                Some(quote!(&#lifetime ()))
            }
            syn::GenericParam::Const(_) => None,
        })
        .collect();
    // the body's own generic arguments: callbacks are inferred from their values
    let turbofish: Vec<TokenStream2> = input
        .sig
        .generics
        .params
        .iter()
        .filter(|param| !matches!(param, syn::GenericParam::Lifetime(_)))
        .map(|param| match param {
            syn::GenericParam::Type(param) if tracked_params.contains(&&param.ident) => quote!(_),
            param => generic_arg(param),
        })
        .collect();

    let tracked: Vec<(&Prop, syn::Ident)> = props
        .iter()
        .filter(|prop| matches!(prop.role, Role::Required | Role::Callback(_)))
        .map(|prop| (prop, format_ident!("__S_{}", prop.name)))
        .collect();
    let state_of = |field: &syn::Ident| tracked.iter().find(|(prop, _)| prop.name == *field).map(|(_, state)| state.clone());
    let states: Vec<&syn::Ident> = tracked.iter().map(|(_, state)| state).collect();
    let unset: Vec<TokenStream2> = tracked
        .iter()
        .map(|(prop, _)| {
            if let Role::Callback(index) = prop.role {
                let unset = &found[index].unset;
                quote!(::std::option::Option<#unset>)
            } else {
                quote!(::bui_charts::__private::Missing)
            }
        })
        .collect();
    let fields: Vec<&syn::Ident> = props.iter().map(|prop| &prop.name).collect();
    let field_types: Vec<TokenStream2> = props
        .iter()
        .map(|prop| match &prop.role {
            Role::Optional(inner) => quote!(::std::option::Option<#inner>),
            Role::Children(item) => quote!(::std::vec::Vec<#item>),
            _ => state_of(&prop.name).to_token_stream(),
        })
        .collect();
    let initial: Vec<TokenStream2> = props
        .iter()
        .map(|prop| match prop.role {
            Role::Required => quote!(::bui_charts::__private::Missing),
            Role::Optional(_) | Role::Callback(_) => quote!(::std::option::Option::None),
            Role::Children(_) => quote!(::std::vec::Vec::new()),
        })
        .collect();

    let mut setters = TokenStream2::new();
    let mut required_traits = TokenStream2::new();
    let mut bounds = Vec::new();
    let mut take_values = TokenStream2::new();
    for prop in &props {
        let field = &prop.name;
        let ty = &prop.ty;
        let set_state = |value: TokenStream2| {
            let states = tracked.iter().map(|(other, state)| if other.name == prop.name { value.clone() } else { state.to_token_stream() });
            quote!(#props_ident<#(#args,)* #(#states,)*>)
        };
        let others: Vec<&&syn::Ident> = fields.iter().filter(|other| **other != field).collect();
        match &prop.role {
            Role::Optional(inner) => {
                let (param, value) = setter_param(inner, field, prop.into);
                setters.extend(quote! {
                    #[must_use]
                    #vis fn #field(mut self, #field: #param) -> Self {
                        self.#field = ::std::option::Option::Some(#value);
                        self
                    }
                });
            }
            Role::Callback(index) => {
                let bounds_of = &found[*index].bounds;
                let next = set_state(quote!(::std::option::Option<__Callback>));
                setters.extend(quote! {
                    #[must_use]
                    #vis fn #field<__Callback: #(#bounds_of)+*>(self, #field: __Callback) -> #next {
                        #props_ident {
                            __marker: ::std::marker::PhantomData,
                            #(#others: self.#others,)*
                            #field: ::std::option::Option::Some(#field),
                        }
                    }
                });
                let state = state_of(field);
                bounds.push(quote!(#state: ::bui_charts::__private::Optional));
                bounds.push(quote!(<#state as ::bui_charts::__private::Optional>::Inner: #(#bounds_of)+*));
                take_values.extend(quote!(let #field = ::bui_charts::__private::Optional::into_option(#field);));
            }
            Role::Required => {
                let (param, value) = setter_param(ty, field, prop.into);
                let next = set_state(quote!(::bui_charts::__private::Set<#ty>));
                setters.extend(quote! {
                    #[must_use]
                    #vis fn #field(self, #field: #param) -> #next {
                        #props_ident {
                            __marker: ::std::marker::PhantomData,
                            #(#others: self.#others,)*
                            #field: ::bui_charts::__private::Set(#value),
                        }
                    }
                });
                let required = format_ident!("__{}Requires_{}", props_ident, field);
                let message = format!("`<{tag}>` is missing the required prop `{field}`");
                let label = format!("add `{field}={{..}}` to this tag");
                required_traits.extend(quote! {
                    #[doc(hidden)]
                    #[allow(non_camel_case_types)]
                    #[diagnostic::on_unimplemented(message = #message, label = #label)]
                    #vis trait #required {
                        type Value;
                        fn value(self) -> Self::Value;
                    }

                    impl<V> #required for ::bui_charts::__private::Set<V> {
                        type Value = V;
                        fn value(self) -> V {
                            self.0
                        }
                    }
                });
                let state = state_of(field);
                bounds.push(quote!(#state: #required<Value = #ty>));
                take_values.extend(quote!(let #field = #required::value(#field);));
            }
            Role::Children(item) => {
                let collect = if is_element(item) {
                    quote!(children)
                } else {
                    quote!(::bui_charts::__private::collect::<#item>(children))
                };
                setters.extend(quote! {
                    #[must_use]
                    #vis fn children(mut self, children: ::std::vec::Vec<::blenderui::element::AnyElement>) -> Self {
                        self.children = #collect;
                        self
                    }
                });
            }
        }
    }

    let finish = if returns_element { quote!(element) } else { quote!(::bui_charts::__private::offer(element)) };
    let mut body = input.clone();
    body.attrs.retain(|attr| !attr.path().is_ident("doc"));
    let docs: Vec<&syn::Attribute> = input.attrs.iter().filter(|attr| attr.path().is_ident("doc")).collect();
    let props_doc = fields.iter().map(ToString::to_string).collect::<Vec<_>>().join(", ");

    Ok(quote! {
        #(#docs)*
        #[allow(clippy::too_many_arguments, clippy::needless_pass_by_value)]
        #body

        #required_traits

        #[doc = concat!("`rsx!` tag: `<", stringify!(#tag), " prop={..} />` builds a [`", stringify!(#props_ident), "`].")]
        #[allow(non_camel_case_types)]
        #vis type #tag<#(#plain),*> = #props_ident<#(#args,)* #(#unset,)*>;

        #[doc = concat!("The props of [`", stringify!(#name), "`]: ", #props_doc)]
        #[allow(non_snake_case, non_camel_case_types, clippy::type_complexity)]
        #vis struct #props_ident<#(#params,)* #(#states,)*> #where_clause {
            __marker: ::std::marker::PhantomData<fn() -> (#(#marker,)*)>,
            #(#fields: #field_types,)*
        }

        #[allow(non_camel_case_types, clippy::new_without_default)]
        impl<#(#params),*> #props_ident<#(#args,)* #(#unset,)*> #where_clause {
            #[must_use]
            #vis fn new() -> Self {
                Self {
                    __marker: ::std::marker::PhantomData,
                    #(#fields: #initial,)*
                }
            }

            #[doc(hidden)]
            #[must_use]
            #vis fn __builder() -> Self {
                Self::new()
            }
        }

        #[allow(non_camel_case_types, clippy::type_complexity)]
        impl<#(#params,)* #(#states,)*> #props_ident<#(#args,)* #(#states,)*> #where_clause {
            #setters

            /// Builds the component now (what `rsx!` does with the tag).
            #vis fn render(self) -> #output
            where
                #(#bounds,)*
            {
                let #props_ident { __marker: _, #(#fields,)* } = self;
                #take_values
                ::blenderui::hooks::component(move || #name::<#(#turbofish),*>(#(#fields),*))
            }
        }

        #[allow(non_camel_case_types)]
        impl<#(#params,)* #(#states,)*> ::blenderui::hooks::Complete for #props_ident<#(#args,)* #(#states,)*>
        where
            #(#where_predicates,)*
            #(#bounds,)*
        {
            type Props = Self;
            fn finish(self) -> Self {
                self
            }
        }

        #[allow(non_camel_case_types)]
        impl<#(#params,)* #(#states,)*> ::blenderui::element::IntoElement for #props_ident<#(#args,)* #(#states,)*>
        where
            #(#where_predicates,)*
            #(#bounds,)*
        {
            fn into_any_element(self) -> ::blenderui::element::AnyElement {
                let element = self.render();
                #finish
            }
        }
    })
}
