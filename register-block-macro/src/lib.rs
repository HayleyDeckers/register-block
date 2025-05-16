//! Procedural macro to generate UART register block and accessors.
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemStruct, Lit};

/// Attribute macro to generate register block and accessors for UART.
#[proc_macro_attribute]
pub fn register_block(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the input struct
    let input = parse_macro_input!(item as ItemStruct);
    let struct_name = &input.ident;
    let fields = if let syn::Fields::Named(fields) = &input.fields {
        &fields.named
    } else {
        return syn::Error::new_spanned(
            &input,
            "register_block only supports structs with named fields",
        )
        .to_compile_error()
        .into();
    };

    // Ensure no generics are specified on the input struct
    if !input.generics.params.is_empty() {
        return syn::Error::new_spanned(
            &input.generics,
            "#[register_block] struct must not have generics; the macro will generate 'T: BaseAddress' automatically."
        ).to_compile_error().into();
    }

    #[allow(non_camel_case_types)]
    #[derive(Clone, Copy)]
    enum Access {
        RW,
        RO,
        WO,
        Clear,
        RC,
    }

    use std::collections::HashMap;
    let mut offset_map: HashMap<u32, Access> = HashMap::new();
    let mut struct_fields = Vec::new();
    for field in fields {
        let field_name = &field.ident;
        let field_ty = &field.ty;
        let mut offset = None;
        let mut access = None;
        let mut doc_attrs = Vec::new();
        for attr in &field.attrs {
            if attr.path().is_ident("doc") {
                doc_attrs.push(attr);
            }
            if attr.path().is_ident("register") {
                let _ = attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("offset") {
                        let lit: Lit = meta.value()?.parse()?;
                        if let Lit::Int(litint) = lit {
                            offset = Some(litint.base10_parse::<u32>().unwrap());
                        }
                    } else if meta.path.is_ident("access") {
                        let lit: Lit = meta.value()?.parse()?;
                        if let Lit::Str(litstr) = lit {
                            let val = litstr.value().to_uppercase();
                            match val.as_str() {
                                "RW" => access = Some(Access::RW),
                                "RO" => access = Some(Access::RO),
                                "WO" => access = Some(Access::WO),
                                "WC" => access = Some(Access::Clear),
                                "RC" => access = Some(Access::RC),
                                _ => panic!(
                                    "Unknown access type: {}. Use RW, RO, WO, WC, or RC.",
                                    val
                                ),
                            }
                        }
                    }
                    Ok(())
                });
            }
        }
        let offset =
            offset.expect("Each register field must have #[register(offset = ..., access = ...)]");
        let access =
            access.expect("Each register field must have #[register(offset = ..., access = ...)]");
        // Overlap check:
        match access {
            Access::RO => {
                if let Some(existing) = offset_map.get(&offset) {
                    match existing {
                        Access::WO | Access::Clear => {} // allowed
                        Access::RW | Access::RO | Access::RC => {
                            return syn::Error::new_spanned(
                                field_name,
                                format!("Duplicate register offset 0x{:X} for RO field {:?}. Only WO/Clear may overlap with RO.", offset, field_name)
                            ).to_compile_error().into();
                        }
                    }
                }
            }
            Access::RW | Access::WO | Access::Clear | Access::RC => {
                if let Some(existing) = offset_map.get(&offset) {
                    match existing {
                        Access::RO => {} // allowed
                        _ => {
                            return syn::Error::new_spanned(
                                field_name,
                                format!("Duplicate register offset 0x{:X} for RW/WO/Clear/RC field {:?}. Only RO may overlap with RW/WO/Clear/RC.", offset, field_name)
                            ).to_compile_error().into();
                        }
                    }
                }
            }
        }
        offset_map.insert(offset, access.clone());
        // Generate accessor function based on access type
        let (ptr_type, init_expr) = match access {
            Access::RW => (
                quote! { ::register_block::RW<#field_ty> },
                quote! { unsafe { ::register_block::RW::new(self.base.base_address() + #offset as usize) } },
            ),
            Access::RO => (
                quote! { ::register_block::RO<#field_ty> },
                quote! { unsafe { ::register_block::RO::new(self.base.base_address() + #offset as usize) } },
            ),
            Access::WO => (
                quote! { ::register_block::WO<#field_ty> },
                quote! { unsafe { ::register_block::WO::new(self.base.base_address() + #offset as usize) } },
            ),
            Access::Clear => (
                quote! { ::register_block::WC<#field_ty> },
                quote! { unsafe { ::register_block::WC::new(self.base.base_address() + #offset as usize) } },
            ),
            Access::RC => (
                quote! { ::register_block::RC<#field_ty> },
                quote! { unsafe { ::register_block::RC::new(self.base.base_address() + #offset as usize) } },
            ),
        };
        let accessor = quote! {
            #(#doc_attrs)*
            #[inline(always)]
            pub fn #field_name(&self) -> #ptr_type {
                #init_expr
            }
        };
        struct_fields.push(accessor);
    }

    let expanded = quote! {
        pub struct #struct_name<T: ::register_block::BaseAddress> {
            base: T,
        }
        impl<T: ::register_block::BaseAddress> #struct_name<T> {
            /// Create a new register block at the given base address.
            pub fn new(base: T) -> Self {
                Self { base }
            }
            #(#struct_fields)*
        }
    };
    TokenStream::from(expanded)
}
