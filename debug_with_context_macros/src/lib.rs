use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::{ToTokens, quote};
use syn::{parse_macro_input, parse_quote, Data, DeriveInput, Field, Fields, GenericParam, Generics, TypeParam, WhereClause, WherePredicate};

/*fn compile_error<T: ToTokens>(tokens: T, message: &'static str) -> proc_macro2::TokenStream {
    syn::Error::new_spanned(tokens, message).to_compile_error()
}*/

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

fn gen_field(field: &Field, field_idx: usize, is_struct: bool, is_named: bool) -> proc_macro2::TokenStream {
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
        if attr.path().is_ident("debug_context") {
            attr.parse_nested_meta(|meta| {
                context_structs.push( meta.path
                        .get_ident()
                        .expect("Expected an identifier for the debug context struct")
                        .clone());;
                Ok(())
            })
            .unwrap();
        }
    }

    /*let context_struct = match context_struct {
        Some(cs) => cs,
        None => {
            return compile_error(ident, "Missing #[debug_context(...)] attribute").into();
        }
    };*/


    let generic_param_types = generics
        .type_params()
        .cloned()
        .collect::<Vec<_>>();


    


    
    
    let output = if context_structs.is_empty(){
        gen_struct_derive(None, &data, &ident, &generic_param_types, &generics)
    } else {
        let mut outputs = Vec::new();

        for c in context_structs {
            outputs.push(gen_struct_derive(Some(c), &data, &ident, &generic_param_types, &generics));
        }

        quote! {
            #(#outputs)*
        } 
    };


    
    let out = output.into();
    //println!("{}", &out);
    out
}


fn gen_struct_derive(context_struct : Option<Ident>, data : &Data, ident : &Ident, generic_param_types : &[TypeParam], generics : &Generics) -> proc_macro2::TokenStream {

    let mut generic_quote = None;

    let mut generic_quote_without_generic_debug_context = None;


    // TODO : make this simpler
    if !generic_param_types.is_empty() {
        let generic_param_context = if context_struct.is_none() { 
            Some(quote! { DEBUG_WITH_CONTEXT_CONTEXT_STRUCT, })
        } else {
            None
        };
        generic_quote = Some(quote! {
            <#generic_param_context #(#generic_param_types,)*>
        });
        generic_quote_without_generic_debug_context = Some(quote! {
            <#(#generic_param_types,)*>
        })
    } else if context_struct.is_none() {
        generic_quote = Some(quote! {
            <DEBUG_WITH_CONTEXT_CONTEXT_STRUCT>
        });
    }

    let mut where_clause = match &generics.where_clause {
        Some(where_clause) => where_clause.clone(), 
        None => WhereClause {
            where_token: Default::default(),
            predicates: syn::punctuated::Punctuated::new(),
        },
    };

    for type_param in &generics.params {
        if let GenericParam::Type(type_param) = type_param {
            let ident = &type_param.ident;
            let type_param_bound: WherePredicate = parse_quote! {
                #ident: DebugWithContext<#context_struct>
            };
            where_clause.predicates.push(type_param_bound);
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
        Data::Struct(s) => {
            match &s.fields {
                Fields::Named(named_fields) => {
                    let named_fields_streams = named_fields.named.iter().enumerate().map(gen_field_struct_named);
                    quote! {
                        f.debug_struct(#ident_lit)
                        #(#named_fields_streams)*
                        .finish()
                    }
                },
                Fields::Unnamed(unnamed_fields) => {
                    let unnamed_field_streams = unnamed_fields.unnamed.iter().enumerate().map(gen_field_struct_unnamed);
                    quote! {
                        f.debug_tuple(#ident_lit)
                        #(#unnamed_field_streams)*
                        .finish()
                    }
                },
                Fields::Unit => {
                    quote! {
                        f.debug_struct(#ident_lit).finish()
                    }
                }
            }
        }
        Data::Union(_) => panic!("Union are not supported for now"),
    };

    let output = if let Some(context_struct) = context_struct {
        quote! {
            #[automatically_derived]
            impl #generic_quote DebugWithContext<#context_struct> for #ident #generic_quote
            #where_clause 
            {
                fn fmt_with_context(&self, f: &mut ::std::fmt::Formatter, context: &#context_struct) -> ::std::fmt::Result {
                    #fmt_code
                }
            }
        }
    } else {
        // not specialized
        quote! {
            #[automatically_derived]
            impl #generic_quote DebugWithContext<DEBUG_WITH_CONTEXT_CONTEXT_STRUCT> for #ident #generic_quote_without_generic_debug_context
            #where_clause 
            {
                fn fmt_with_context(&self, f: &mut ::std::fmt::Formatter, context: &DEBUG_WITH_CONTEXT_CONTEXT_STRUCT) -> ::std::fmt::Result {
                    #fmt_code
                }
            }
        }
    };
    output
}