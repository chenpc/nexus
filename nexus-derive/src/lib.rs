use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Attribute, FnArg, ImplItem, ItemImpl, Pat, ReturnType};

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

                // Collect parameter names and types (skip &self).
                let mut param_names = Vec::new();
                let mut param_name_strings = Vec::new();

                for arg in method.sig.inputs.iter().skip(1) {
                    if let FnArg::Typed(pat_type) = arg {
                        if let Pat::Ident(pat_ident) = &*pat_type.pat {
                            let name = &pat_ident.ident;
                            param_names.push(name.clone());
                            param_name_strings.push(name.to_string());
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
                    nexus::CommandInfo {
                        name: #method_name_str.to_string(),
                        args: vec![#(#param_name_strings.to_string()),*],
                        description: #doc.to_string(),
                    }
                });

                // Rebuild method without #[command] attribute.
                let remaining_attrs = strip_command_attr(&method.attrs);
                let vis = &method.vis;
                let sig = &method.sig;
                let block = &method.block;
                let has_return = !matches!(&sig.output, ReturnType::Default);
                if has_return {
                    cleaned_methods.push(quote! {
                        #(#remaining_attrs)*
                        #vis #sig #block
                    });
                } else {
                    cleaned_methods.push(quote! {
                        #(#remaining_attrs)*
                        #vis #sig #block
                    });
                }
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
        impl nexus::Service for #self_ty {
            fn name(&self) -> &str {
                #service_name
            }

            fn commands(&self) -> Vec<nexus::CommandInfo> {
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
