use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::{ToTokens, quote};
use syn::{
    Data, DeriveInput, Field, Fields, GenericParam, Generics, Token, Type, WhereClause,
    WherePredicate, parse::Parse, parse_macro_input, parse_quote, punctuated::Punctuated,
    token::Comma,
};

fn gen_field_struct_named(field: (usize, &Field)) -> proc_macro2::TokenStream {
    gen_field(field.1, field.0, true, true)
}

fn gen_field_enum_named(field: (usize, &Field)) -> proc_macro2::TokenStream {
    gen_field(field.1, field.0, false, true)
}

fn gen_field_struct_unnamed(field: (usize, &Field)) -> proc_macro2::TokenStream {
    gen_field(field.1, field.0, true, false)
}

fn gen_field_enum_unnamed(field: (usize, &Field)) -> proc_macro2::TokenStream {
    gen_field(field.1, field.0, false, false)
}

fn gen_field_do_nothing(_field: (usize, &Field)) -> proc_macro2::TokenStream {
    quote! {}
}

fn get_unnamed_enum_arg(idx: usize) -> String {
    "arg".to_string() + &idx.to_string()
}

// TODO : can I ask to the compiler if type implements Debug and in this case use field and not field_with ?

fn gen_field(
    field: &Field,
    field_idx: usize,
    is_struct: bool,
    is_named: bool,
) -> proc_macro2::TokenStream {
    let field_name = match &field.ident {
        Some(i) => {
            quote! { #i }
        }
        None => {
            if is_struct {
                syn::Index::from(field_idx).into_token_stream()
            } else {
                let idx_str = get_unnamed_enum_arg(field_idx);
                Ident::new(&idx_str, proc_macro2::Span::call_site()).into_token_stream()
            }
        }
    };

    let field_name_str = field_name.to_string();
    let field_name_lit = syn::LitStr::new(&field_name_str, proc_macro2::Span::call_site());

    let obj_access = if is_struct {
        quote! {
            self. #field_name
        }
    } else {
        quote! {
            #field_name
        }
    };

    let optional_name_arg = if is_named {
        Some(quote! { #field_name_lit, })
    } else {
        None
    };

    quote! {
        .field_with(#optional_name_arg  |fmt| {
            #obj_access .fmt_with_context(fmt, context)
        })
    }
}

enum EnumType {
    Struct,
    Tuple,
    Empty,
}

#[proc_macro_derive(DebugWithContext, attributes(debug_context))]
pub fn derive(input: TokenStream) -> TokenStream {
    let DeriveInput {
        ident,
        attrs,
        vis: _,
        generics,
        data,
    } = parse_macro_input!(input);
    let mut context_structs = Vec::new();
    for attr in attrs {
        if !attr.path().is_ident("debug_context") {
            continue;
        }
        match attr.parse_args() {
            Ok(s) => {
                context_structs.push(s);
            }
            Err(e) => return e.into_compile_error().into(),
        }
    }

    let output = if context_structs.is_empty() {
        gen_struct_derive(None, &data, &ident, &generics)
    } else {
        let mut outputs = Vec::new();

        for c in context_structs {
            outputs.push(gen_struct_derive(Some(c), &data, &ident, &generics));
        }

        quote! {
            #(#outputs)*
        }
    };

    output.into()
}

struct ContextAttr {
    impl_generics: Option<Punctuated<GenericParam, Comma>>,
    ty: Type,
    where_clause: Option<WhereClause>,
}

impl Parse for ContextAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut impl_generics = None;
        if input.parse::<Token![<]>().is_ok() {
            let impl_generics = impl_generics.insert(Punctuated::new());
            while !input.peek(Token![>]) {
                impl_generics.push(input.parse()?);
                if input.peek(Token![>]) {
                    break;
                }
                input.parse::<Token![,]>()?;
            }
            input.parse::<Token![>]>()?;
        }
        let ty = input.parse()?;
        let where_clause = input.parse().ok();

        Ok(Self {
            impl_generics,
            ty,
            where_clause,
        })
    }
}

fn gen_struct_derive(
    context_attr: Option<ContextAttr>,
    data: &Data,
    ident: &Ident,
    generics: &Generics,
) -> proc_macro2::TokenStream {
    let mut impl_generics = generics.params.clone();
    let mut where_predicates = generics
        .where_clause
        .clone()
        .map_or_else(Default::default, |w| w.predicates);
    let context_ty;

    if let Some(context_attr) = context_attr {
        impl_generics.extend(context_attr.impl_generics.into_iter().flatten());
        where_predicates.extend(
            context_attr
                .where_clause
                .map(|w| w.predicates)
                .into_iter()
                .flatten(),
        );
        context_ty = context_attr.ty;
    } else {
        impl_generics.push(parse_quote! { DEBUG_WITH_CONTEXT_CONTEXT_STRUCT });
        context_ty = parse_quote! { DEBUG_WITH_CONTEXT_CONTEXT_STRUCT };
    }

    for type_param in &generics.params {
        if let GenericParam::Type(type_param) = type_param {
            let ident = &type_param.ident;
            let type_param_bound: WherePredicate = parse_quote! {
                #ident: DebugWithContext<#context_ty>
            };
            where_predicates.push(type_param_bound);
        }
    }

    let ident_str = ident.to_string();
    let ident_lit = syn::LitStr::new(&ident_str, proc_macro2::Span::call_site());
    let fmt_code = match data {
        Data::Enum(e) => {
            let variants = e.variants.iter().map(|v|{
                let variant_name = &v.ident;
                let variant_name_str= variant_name.to_string();
                let variant_name_lit = syn::LitStr::new(&variant_name_str, proc_macro2::Span::call_site());

                let is_tuple = v.fields.iter().any(|e| e.ident.is_none());
                let is_empty = matches!(v.fields, Fields::Unit);

                let enum_type = if is_empty {
                    EnumType::Empty
                } else if is_tuple {
                    EnumType::Tuple
                } else {
                    EnumType::Struct
                };
                let gen_field_enum = match enum_type {
                    EnumType::Tuple => gen_field_enum_unnamed,
                    EnumType::Struct => gen_field_enum_named,
                    EnumType::Empty => gen_field_do_nothing,
                };
                let variant_fields = v.fields.iter().enumerate().map(gen_field_enum);
                match enum_type {
                    EnumType::Tuple => {
                        let variant_field_names_lit = (0..v.fields.len()).map(get_unnamed_enum_arg).map(|e| Ident::new(&e, proc_macro2::Span::call_site()));
                            quote! {
                                Self:: #variant_name ( #(#variant_field_names_lit,)* ) => f.debug_tuple(#variant_name_lit)
                                                #(#variant_fields)* .finish(),
                            }
                    }
                    EnumType::Struct => {
                        let variant_field_names = v.fields.iter().map(|f| f.ident.as_ref());
                        quote! {
                            Self:: #variant_name { #(#variant_field_names,)* } => f.debug_struct(#variant_name_lit)
                                                #(#variant_fields)* .finish() ,
                        }
                    }
                    EnumType::Empty => {
                        quote! {
                            Self:: #variant_name => write!(f, #variant_name_lit),
                        }
                    }
                }
            });

            quote! {
                match self {
                    #(#variants)*
                }
            }
        }
        Data::Struct(s) => match &s.fields {
            Fields::Named(named_fields) => {
                let named_fields_streams = named_fields
                    .named
                    .iter()
                    .enumerate()
                    .map(gen_field_struct_named);
                quote! {
                    f.debug_struct(#ident_lit)
                    #(#named_fields_streams)*
                    .finish()
                }
            }
            Fields::Unnamed(unnamed_fields) => {
                let unnamed_field_streams = unnamed_fields
                    .unnamed
                    .iter()
                    .enumerate()
                    .map(gen_field_struct_unnamed);
                quote! {
                    f.debug_tuple(#ident_lit)
                    #(#unnamed_field_streams)*
                    .finish()
                }
            }
            Fields::Unit => {
                quote! {
                    f.debug_struct(#ident_lit).finish()
                }
            }
        },
        Data::Union(_) => panic!("Union are not supported for now"),
    };

    let impl_generic_quote = if impl_generics.is_empty() {
        None
    } else {
        Some(quote! { <#impl_generics> })
    };
    let (_, ty_generics, _) = generics.split_for_impl();
    let where_clause = WhereClause {
        where_token: Default::default(),
        predicates: where_predicates,
    };

    quote! {
        #[automatically_derived]
        impl #impl_generic_quote ::debug_with_context::DebugWithContext<#context_ty> for #ident #ty_generics
        #where_clause
        {
            fn fmt_with_context(&self, f: &mut ::std::fmt::Formatter, context: &#context_ty) -> ::std::fmt::Result {
                #fmt_code
            }
        }
    }
}
