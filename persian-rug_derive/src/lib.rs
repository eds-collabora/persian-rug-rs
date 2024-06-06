use proc_macro::{self, TokenStream};
use proc_macro2 as pm2;
use quote::ToTokens;

enum ConstraintItem {
    Context(syn::Ident),
    Access(Vec<syn::Type>),
}

impl syn::parse::Parse for ConstraintItem {
    fn parse(input: syn::parse::ParseStream<'_>) -> syn::Result<Self> {
        let attr: syn::Ident = input.parse()?;
        match attr.to_string().as_str() {
            "context" => {
                let _: syn::Token![=] = input.parse()?;
                let value = input.parse()?;
                Ok(ConstraintItem::Context(value))
            }
            "access" => {
                let content;
                let _: syn::token::Paren = syn::parenthesized!(content in input);
                let punc =
                    syn::punctuated::Punctuated::<syn::Type, syn::Token![,]>::parse_terminated(
                        &content,
                    )?;
                Ok(ConstraintItem::Access(punc.into_iter().collect()))
            }
            _ => Err(syn::Error::new_spanned(
                attr,
                "unsupported persian-rug constraint",
            )),
        }
    }
}

struct ConstraintArgs {
    pub context: syn::Ident,
    pub used_types: Vec<syn::Type>,
}

impl syn::parse::Parse for ConstraintArgs {
    fn parse(input: syn::parse::ParseStream<'_>) -> syn::Result<Self> {
        let punc =
            syn::punctuated::Punctuated::<ConstraintItem, syn::Token![,]>::parse_terminated(input)?;
        let mut context = None;
        let mut used_types = Vec::new();

        for item in punc.into_iter() {
            match item {
                ConstraintItem::Context(id) => {
                    context = Some(id);
                }
                ConstraintItem::Access(tys) => {
                    used_types.extend(tys);
                }
            }
        }

        context
            .map(|context| Self {
                context,
                used_types,
            })
            .ok_or_else(|| {
                syn::Error::new(
                    pm2::Span::call_site(),
                    "No context provided for constraints.",
                )
            })
    }
}

/// Add the type constraints necessary for an impl using persian-rug.
///
/// Rust currently requires all relevant constraints to be written out
/// for every impl using a given type. For persian-rug in particular,
/// there are typically many constraints of a simple kind: for every
/// type owned by the given `Context`, there must be an `Owner`
/// implementation for the context and there must be a matching
/// `Contextual` implementation for the type. This macro simply
/// generates these constraints for you.
///
/// The attribute takes two types of argument:
/// - `context` specifies the name of the type of the context.
/// - `access(...)` specifies the types that this impl requires to
///   exist within that context. Typically each type requires some
///   other types to also exist in its context for it to be
///   well-formed.  This argument needs to be given the transitive
///   closure of all such types, both direct and indirect dependencies
///   of the impl itself. It is unfortunately not possible at present
///   to find the indirect dependencies automatically.
///
/// Example:
/// ```rust
/// use persian_rug::{contextual, Context, Mutator, Proxy};
///
/// #[contextual(C)]
/// struct Foo<C: Context> {
///    _marker: core::marker::PhantomData<C>,
///    a: i32
/// }
///
/// struct Bar<C: Context> {
///    foo: Proxy<Foo<C>>
/// }
///
/// #[persian_rug::constraints(context = C, access(Foo<C>))]
/// impl<C> Bar<C> {
///    pub fn new<M: Mutator<Context=C>>(foo: Foo<C>, mut mutator: M) -> Self {
///        Self { foo: mutator.add(foo) }
///    }
/// }
/// ```
#[proc_macro_attribute]
pub fn constraints(args: TokenStream, input: TokenStream) -> TokenStream {
    let mut target: syn::Item = syn::parse_macro_input!(input);

    let generics = match &mut target {
        syn::Item::Enum(e) => &mut e.generics,
        syn::Item::Fn(f) => &mut f.sig.generics,
        syn::Item::Impl(i) => &mut i.generics,
        syn::Item::Struct(s) => &mut s.generics,
        syn::Item::Trait(t) => &mut t.generics,
        syn::Item::TraitAlias(t) => &mut t.generics,
        syn::Item::Type(t) => &mut t.generics,
        syn::Item::Union(u) => &mut u.generics,
        _ => {
            return syn::Error::new(
                pm2::Span::call_site(),
                "This attribute extends a where clause, or generic constraints. It cannot be used here."
            )
                .to_compile_error()
                .into();
        }
    };

    let ConstraintArgs {
        context,
        used_types,
    } = syn::parse_macro_input!(args);

    let wc = generics.make_where_clause();

    let mut getters = syn::punctuated::Punctuated::<syn::TypeParamBound, syn::token::Add>::new();
    getters.push(syn::parse_quote! { ::persian_rug::Context });
    for ty in &used_types {
        getters.push(syn::parse_quote! { ::persian_rug::Owner<#ty> });
    }

    wc.predicates.push(syn::parse_quote! {
        #context: #getters
    });

    for ty in &used_types {
        wc.predicates.push(syn::parse_quote! {
            #ty: ::persian_rug::Contextual<Context = #context>
        });
    }

    target.into_token_stream().into()
}

/// Convert an annotated struct into a `Context`
///
/// Each field marked with `#[table]` will be converted to be a
/// `Table` of values of the same type. An implementation of `Context`
/// will be provided. In addition, an implementation of `Owner` for
/// each field type will be derived for the overall struct.
///
/// Note that a `Context` can only contain one table of each type.
///
/// Example:
/// ```rust
/// use persian_rug::{contextual, persian_rug, Proxy};
///
/// #[contextual(MyRug)]
/// struct Foo {
///    a: i32
/// }
///
/// #[contextual(MyRug)]
/// struct Bar {
///    a: i32,
///    b: Proxy<Foo>
/// };
///
/// #[persian_rug]
/// struct MyRug(#[table] Foo, #[table] Bar);
/// ```
#[proc_macro_attribute]
pub fn persian_rug(_args: TokenStream, input: TokenStream) -> TokenStream {
    let syn::DeriveInput {
        attrs,
        vis,
        ident: ty_ident,
        data,
        generics,
    } = syn::parse_macro_input!(input);

    let (generics, ty_generics, wc) = generics.split_for_impl();

    let mut impls = pm2::TokenStream::new();

    let body = if let syn::Data::Struct(s) = data {
        let mut fields = syn::punctuated::Punctuated::<syn::Field, syn::Token![,]>::new();

        let mut process_field = |field: &syn::Field| {
            let is_table = field.attrs.iter().any(|attr| attr.path.is_ident("table"));

            let field_type = &field.ty;
            let ident = field
                .ident
                .as_ref()
                .map(|id| syn::Member::Named(id.clone()))
                .unwrap_or_else(|| {
                    syn::Member::Unnamed(syn::Index {
                        index: fields.len() as u32,
                        span: pm2::Span::call_site(),
                    })
                });

            let vis = &field.vis;

            let attrs = field
                .attrs
                .iter()
                .filter(|a| !a.path.is_ident("table"))
                .cloned()
                .collect::<Vec<_>>();

            if !is_table {
                fields.push(field.clone());
            } else {
                fields.push(syn::Field {
                    attrs,
                    vis: vis.clone(),
                    ident: if let syn::Member::Named(id) = &ident {
                        Some(id.clone())
                    } else {
                        None
                    },
                    colon_token: field.colon_token,
                    ty: syn::parse_quote! {
                        ::persian_rug::Table<#field_type>
                    },
                });

                impls.extend(quote::quote! {
                    impl #generics ::persian_rug::Owner<#field_type> for #ty_ident #ty_generics #wc {
                        fn add(&mut self, what: #field_type) -> ::persian_rug::Proxy<#field_type> {
                            self.#ident.push(what)
                        }
                        fn get(&self, what: &::persian_rug::Proxy<#field_type>) -> &#field_type {
                            self.#ident.get(what).unwrap()
                        }
                        fn get_mut(&mut self, what: &::persian_rug::Proxy<#field_type>) -> &mut #field_type {
                            self.#ident.get_mut(what).unwrap()
                        }
                        fn get_iter(&self) -> ::persian_rug::TableIterator<'_, #field_type> {
                            self.#ident.iter()
                        }
                        fn get_iter_mut(&mut self) -> ::persian_rug::TableMutIterator<'_, #field_type> {
                            self.#ident.iter_mut()
                        }
                        fn get_proxy_iter(&self) -> ::persian_rug::TableProxyIterator<'_, #field_type> {
                            self.#ident.iter_proxies()
                        }
                    }
                });
            }
        };

        match s.fields {
            syn::Fields::Named(syn::FieldsNamed { named, .. }) => {
                for field in named.iter() {
                    (process_field)(field);
                }
                quote::quote! {
                    #vis struct #ty_ident #generics #wc {
                        #fields
                    }
                }
            }
            syn::Fields::Unnamed(syn::FieldsUnnamed { unnamed, .. }) => {
                for field in unnamed.iter() {
                    (process_field)(field);
                }
                quote::quote! {
                    #vis struct #ty_ident #generics #wc(
                        #fields
                    );
                }
            }
            syn::Fields::Unit => {
                quote::quote! {
                    #vis struct #ty_ident #generics #wc;
                }
            }
        }
    } else {
        return syn::Error::new(
            pm2::Span::call_site(),
            "Only structs can be annotated as persian-rugs.",
        )
        .to_compile_error()
        .into();
    };

    let attrs = {
        let mut res = pm2::TokenStream::new();
        for attr in attrs {
            attr.to_tokens(&mut res);
        }
        res
    };

    let res = quote::quote! {
        #attrs
        #body

        impl #generics ::persian_rug::Context for #ty_ident #ty_generics #wc {
            fn add<T>(&mut self, what: T) -> ::persian_rug::Proxy<T>
            where
                #ty_ident #ty_generics: ::persian_rug::Owner<T>,
                T: ::persian_rug::Contextual<Context=Self>
            {
                <Self as ::persian_rug::Owner<T>>::add(self, what)
            }

            fn get<T>(&self, what: &::persian_rug::Proxy<T>) -> &T
            where
                #ty_ident #ty_generics: ::persian_rug::Owner<T>,
                T: ::persian_rug::Contextual<Context=Self>
            {
                <Self as ::persian_rug::Owner<T>>::get(self, what)
            }

            fn get_mut<T>(&mut self, what: &::persian_rug::Proxy<T>) -> &mut T
            where
                #ty_ident #ty_generics: ::persian_rug::Owner<T>,
                T: ::persian_rug::Contextual<Context=Self>
            {
                <Self as ::persian_rug::Owner<T>>::get_mut(self, what)
            }

            fn get_iter<T>(&self) -> ::persian_rug::TableIterator<'_, T>
            where
                #ty_ident #ty_generics: ::persian_rug::Owner<T>,
                T: ::persian_rug::Contextual<Context=Self>
            {
                <Self as ::persian_rug::Owner<T>>::get_iter(self)
            }

            fn get_iter_mut<T>(&mut self) -> ::persian_rug::TableMutIterator<'_, T>
            where
                #ty_ident #ty_generics: ::persian_rug::Owner<T>,
                T: ::persian_rug::Contextual<Context=Self>
            {
                <Self as ::persian_rug::Owner<T>>::get_iter_mut(self)
            }

            fn get_proxy_iter<T>(&self) -> ::persian_rug::TableProxyIterator<'_, T>
            where
                #ty_ident #ty_generics: ::persian_rug::Owner<T>,
                T: ::persian_rug::Contextual<Context=Self>
            {
                <Self as ::persian_rug::Owner<T>>::get_proxy_iter(self)
            }
        }

        #impls
    };

    res.into()
}

/// Provide a implementation of `Contextual` for a type.
///
/// This is a very simple derive-style macro, that creates an
/// impl for `Contextual` for the type it annotates. It takes
/// one argument, which is the `Context` type that this
/// type belongs to.
///
/// Example:
/// ```rust
/// use persian_rug::{contextual, Context};
///
/// #[contextual(C)]
/// struct Foo<C: Context> {
///    _marker: core::marker::PhantomData<C>
/// }
/// ```
/// which is equivalent to the following:
/// ```rust
/// use persian_rug::{Context, Contextual};
///
/// struct Foo<C: Context> {
///    _marker: core::marker::PhantomData<C>
/// }
///
/// impl<C: Context> Contextual for Foo<C> {
///    type Context = C;
/// }
/// ```
#[proc_macro_attribute]
pub fn contextual(args: TokenStream, input: TokenStream) -> TokenStream {
    let body = pm2::TokenStream::from(input.clone());

    let syn::DeriveInput {
        ident, generics, ..
    } = syn::parse_macro_input!(input);

    if args.is_empty() {
        return syn::Error::new(
            pm2::Span::call_site(),
            "You must specify the associated context when using contextual.",
        )
        .to_compile_error()
        .into();
    }

    let context: syn::Type = syn::parse_macro_input!(args);

    let (generics, ty_generics, wc) = generics.split_for_impl();

    let res = quote::quote! {
        #body

        impl #generics ::persian_rug::Contextual for #ident #ty_generics #wc {
            type Context = #context;
        }
    };

    res.into()
}
