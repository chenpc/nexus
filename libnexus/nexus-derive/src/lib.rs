use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Attribute, FnArg, ImplItem, ItemImpl, Pat};

/// Extract doc comment strings from attributes.
fn extract_doc_comment(attrs: &[Attribute]) -> String {
    attrs
        .iter()
        .filter_map(|attr| {
            if attr.path().is_ident("doc") {
                if let syn::Meta::NameValue(nv) = &attr.meta {
                    if let syn::Expr::Lit(expr_lit) = &nv.value {
                        if let syn::Lit::Str(s) = &expr_lit.lit {
                            return Some(s.value().trim().to_string());
                        }
                    }
                }
            }
            None
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Check if an attribute list contains `#[command]`.
fn has_command_attr(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| attr.path().is_ident("command"))
}

/// Remove `#[command]` attributes from the list, returning only non-command attrs.
fn strip_command_attr(attrs: &[Attribute]) -> Vec<&Attribute> {
    attrs
        .iter()
        .filter(|attr| !attr.path().is_ident("command"))
        .collect()
}

/// Parse `#[arg(hint = "...", complete = "...")]` from parameter attributes.
fn parse_arg_attr(attrs: &[Attribute]) -> (String, String) {
    let mut hint = String::new();
    let mut completer = String::new();

    for attr in attrs {
        if attr.path().is_ident("arg") {
            let _ = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("hint") {
                    let value = meta.value()?;
                    let lit: syn::LitStr = value.parse()?;
                    hint = lit.value();
                } else if meta.path.is_ident("complete") {
                    let value = meta.value()?;
                    let lit: syn::LitStr = value.parse()?;
                    completer = lit.value();
                }
                Ok(())
            });
        }
    }

    (hint, completer)
}

/// Strip `#[arg(...)]` attributes from a function signature's parameters.
fn strip_arg_attrs(sig: &syn::Signature) -> syn::Signature {
    let mut sig = sig.clone();
    for input in &mut sig.inputs {
        if let FnArg::Typed(pat_type) = input {
            pat_type.attrs.retain(|attr| !attr.path().is_ident("arg"));
        }
    }
    sig
}

#[proc_macro_attribute]
pub fn nexus_service(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemImpl);

    // Extract the struct name from the impl block.
    let self_ty = &input.self_ty;
    let struct_name = quote!(#self_ty).to_string();
    let service_name = struct_name.to_lowercase();

    let mut command_infos = Vec::new();
    let mut match_arms = Vec::new();
    let mut cleaned_methods = Vec::new();

    for item in &input.items {
        if let ImplItem::Fn(method) = item {
            if has_command_attr(&method.attrs) {
                let method_name = &method.sig.ident;
                let method_name_str = method_name.to_string();
                let doc = extract_doc_comment(&method.attrs);

                // Collect parameter names, hints, and completers (skip &self).
                let mut param_names = Vec::new();
                let mut param_name_strings = Vec::new();
                let mut param_hints = Vec::new();
                let mut param_completers = Vec::new();

                for arg in method.sig.inputs.iter().skip(1) {
                    if let FnArg::Typed(pat_type) = arg {
                        if let Pat::Ident(pat_ident) = &*pat_type.pat {
                            let name = &pat_ident.ident;
                            let (hint, completer) = parse_arg_attr(&pat_type.attrs);
                            param_names.push(name.clone());
                            param_name_strings.push(name.to_string());
                            param_hints.push(hint);
                            param_completers.push(completer);
                        }
                    }
                }

                let num_params = param_names.len();

                // Generate the match arm for execute dispatch.
                // Each parameter is extracted positionally from the args Vec<String>.
                let param_extractions: Vec<_> = param_names
                    .iter()
                    .enumerate()
                    .map(|(i, name)| {
                        quote! {
                            let #name = args.get(#i)
                                .ok_or_else(|| anyhow::anyhow!(
                                    "missing argument '{}' (expected {} args)",
                                    stringify!(#name),
                                    #num_params
                                ))?
                                .clone();
                        }
                    })
                    .collect();

                match_arms.push(quote! {
                    #method_name_str => {
                        #(#param_extractions)*
                        self.#method_name(#(#param_names),*).await
                    }
                });

                command_infos.push(quote! {
                    libnexus::CommandInfo {
                        name: #method_name_str.to_string(),
                        args: vec![#(libnexus::ArgInfo {
                            name: #param_name_strings.to_string(),
                            hint: #param_hints.to_string(),
                            completer: #param_completers.to_string(),
                        }),*],
                        description: #doc.to_string(),
                    }
                });

                // Rebuild method without #[command] and #[arg] attributes.
                let remaining_attrs = strip_command_attr(&method.attrs);
                let vis = &method.vis;
                let sig = strip_arg_attrs(&method.sig);
                let block = &method.block;
                cleaned_methods.push(quote! {
                    #(#remaining_attrs)*
                    #vis #sig #block
                });
            } else {
                // Non-command methods pass through unchanged.
                cleaned_methods.push(quote! { #item });
            }
        } else {
            cleaned_methods.push(quote! { #item });
        }
    }

    let (impl_generics, _, where_clause) = input.generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics #self_ty #where_clause {
            #(#cleaned_methods)*
        }

        #[async_trait::async_trait]
        impl libnexus::Service for #self_ty {
            fn name(&self) -> &str {
                #service_name
            }

            fn commands(&self) -> Vec<libnexus::CommandInfo> {
                vec![#(#command_infos),*]
            }

            async fn execute(&self, action: &str, args: Vec<String>) -> anyhow::Result<String> {
                match action {
                    #(#match_arms,)*
                    _ => Err(anyhow::anyhow!("unknown command '{}'", action)),
                }
            }
        }
    };

    TokenStream::from(expanded)
}
