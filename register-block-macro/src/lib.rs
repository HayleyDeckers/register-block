//! Procedural macro to generate UART register block and accessors.
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{ItemStruct, Lit, parse_macro_input};

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

    // Define Access enum inside the macro function to avoid polluting user namespace
    #[allow(non_camel_case_types)]
    #[derive(Clone, Copy)]
    enum Access {
        RW,
        RO,
        WO,
        Clear,
    }

    // Collect offsets and access types for overlap checking
    let mut accessors = Vec::new();
    use std::collections::HashMap;
    let mut offset_map: HashMap<u32, Access> = HashMap::new();
    for field in fields {
        let field_name = &field.ident;
        let field_ty = &field.ty;
        let mut offset = None;
        let mut access = None;
        let mut doc_attrs = Vec::new();
        // Parse doc and custom attributes for offset and access type
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
                                "CLEAR" | "Clear" => access = Some(Access::Clear),
                                _ => panic!(
                                    "Unknown access type: {}. Use RW, RO, WO, or Clear.",
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
                        Access::RW | Access::RO => {
                            return syn::Error::new_spanned(
                                field_name,
                                format!("Duplicate register offset 0x{:X} for RO field {:?}. Only WO/Clear may overlap with RO.", offset, field_name)
                            ).to_compile_error().into();
                        }
                    }
                }
            }
            Access::RW | Access::WO | Access::Clear => {
                if let Some(existing) = offset_map.get(&offset) {
                    match existing {
                        Access::RO => {} // allowed
                        _ => {
                            return syn::Error::new_spanned(
                                field_name,
                                format!("Duplicate register offset 0x{:X} for RW/WO/Clear field {:?}. Only RO may overlap with RW/WO/Clear.", offset, field_name)
                            ).to_compile_error().into();
                        }
                    }
                }
            }
        }
        // Record the offset and access type for future checks
        offset_map.insert(offset, access.clone());
        // Generate accessors based on access type, propagating doc comments
        let getter = match access {
            Access::RW | Access::RO => {
                let method = format_ident!("read_{}", field_name.as_ref().unwrap());
                Some(quote! {
                    #(#doc_attrs)*
                    #[inline(always)]
                    pub fn #method(&self) -> #field_ty {
                        unsafe {
                            ((self.base.base_address() as *const u8).add(#offset as usize) as *const #field_ty).read_volatile()
                        }
                    }
                })
            }
            Access::WO | Access::Clear => None,
        };
        let setter = match access {
            Access::RW | Access::WO => {
                let method = format_ident!("write_{}", field_name.as_ref().unwrap());
                Some(quote! {
                    #(#doc_attrs)*
                    #[inline(always)]
                    pub fn #method(&self, value: #field_ty) {
                        unsafe {
                            ((self.base.base_address() as *mut u8).add(#offset as usize) as *mut #field_ty).write_volatile(value)
                        }
                    }
                })
            }
            Access::Clear => {
                let method = format_ident!("clear_{}", field_name.as_ref().unwrap());
                Some(quote! {
                    #(#doc_attrs)*
                    #[inline(always)]
                    pub fn #method(&self) {
                        unsafe {
                            ((self.base.base_address() as *mut u8).add(#offset as usize) as *mut #field_ty).write_volatile(Default::default())
                        }
                    }
                })
            }
            Access::RO => None,
        };
        if let Some(getter) = getter {
            accessors.push(getter);
        }
        if let Some(setter) = setter {
            accessors.push(setter);
        }
    }

    // Add a base field of type T: BaseAddress and implement generics
    let expanded = quote! {
        pub struct #struct_name<T: ::register_block::BaseAddress> {
            base: T,
        }
        impl<T: ::register_block::BaseAddress> #struct_name<T> {
            /// Create a new register block at the given base address.
            pub fn new(base: T) -> Self {
                Self { base }
            }
            #(#accessors)*
        }
    };
    TokenStream::from(expanded)
}
