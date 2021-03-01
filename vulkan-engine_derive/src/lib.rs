#![feature(proc_macro_diagnostic)]

use proc_macro::{Diagnostic, TokenStream};
use quote::quote_spanned;
use syn::{spanned::Spanned, DeriveInput};

#[proc_macro_derive(MaterialBindingFragment)]
pub fn derive_material_binding_fragment(target: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(target).expect("Failed to parse");

    if let syn::Data::Struct(_) = &ast.data {
        let name = &ast.ident;

        let gen = quote_spanned! {ast.span()=>
            impl ::vulkan_engine::scene::material::MaterialBinding for #name {
                fn get_material_binding() -> ::vulkan_engine::scene::material::MaterialDataBinding {
                    ::vulkan_engine::scene::material::MaterialDataBinding {
                        binding_type: ::vulkan_engine::scene::material::MaterialDataBindingType::Uniform,
                        binding_stage: ::vulkan_engine::scene::material::MaterialDataBindingStage::Fragment,
                    }
                }
                fn get_material_resource_helper(&self) -> ::vulkan_engine::scene::material::MaterialResourceHelper {
                    ::vulkan_engine::scene::material::MaterialResourceHelper::UniformBuffer(
                        unsafe { ::std::slice::from_raw_parts(self as *const #name as *const u8, size_of::<Self>()) }
                    )
                }
            }
        };
        gen.into()
    } else {
        Diagnostic::spanned(
            ast.span().unwrap(),
            proc_macro::Level::Error,
            "#[derive(MaterialBindingFragment)] can only be used on structs",
        )
        .emit();
        TokenStream::new()
    }
}

#[proc_macro_derive(MaterialBindingVertex)]
pub fn derive_material_binding_vertex(target: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(target).expect("Failed to parse");

    if let syn::Data::Struct(_) = &ast.data {
        let name = &ast.ident;

        let gen = quote_spanned! {ast.span()=>
            impl ::vulkan_engine::scene::material::MaterialBinding for #name {
                fn get_material_binding() -> ::vulkan_engine::scene::material::MaterialDataBinding {
                    ::vulkan_engine::scene::material::MaterialDataBinding {
                        binding_type: ::vulkan_engine::scene::material::MaterialDataBindingType::Uniform,
                        binding_stage: ::vulkan_engine::scene::material::MaterialDataBindingStage::Vertex,
                    }
                }
                fn get_material_resource_helper(&self) -> ::vulkan_engine::scene::material::MaterialResourceHelper {
                    ::vulkan_engine::scene::material::MaterialResourceHelper::UniformBuffer(
                        unsafe { ::std::slice::from_raw_parts(self as *const #name as *const u8, size_of::<Self>()) }
                    )
                }
            }
        };
        gen.into()
    } else {
        Diagnostic::spanned(
            ast.span().unwrap(),
            proc_macro::Level::Error,
            "#[derive(MaterialBindingFragment)] can only be used on structs",
        )
        .emit();
        TokenStream::new()
    }
}

#[proc_macro_derive(MaterialData)]
pub fn derive_material_data(target: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(target).expect("Failed to parse");

    if let syn::Data::Struct(struct_data) = &ast.data {
        let name = &ast.ident;

        let mut gen_bindings = proc_macro2::TokenStream::new();
        let mut gen_helpers = proc_macro2::TokenStream::new();
        for f in &struct_data.fields {
            if let Some(name) = &f.ident {
                let type_ident = &f.ty;

                let gen_field = quote_spanned! {f.span()=>
                    <#type_ident as MaterialBinding>::get_material_binding(),
                };
                gen_bindings.extend(gen_field);

                let gen_helper = quote_spanned! {f.span()=>
                    self.#name.get_material_resource_helper(),
                };
                gen_helpers.extend(gen_helper);
            }
        }

        let gen = quote_spanned! {ast.span()=>
            impl ::vulkan_engine::scene::material::MaterialData for #name {
                fn get_material_layout() -> ::vulkan_engine::scene::material::MaterialDataLayout {
                    ::vulkan_engine::scene::material::MaterialDataLayout {
                        bindings: vec! [
                            #gen_bindings
                        ]
                    }
                }
                fn get_material_resource_helpers(&self) -> Vec<::vulkan_engine::scene::material::MaterialResourceHelper> {
                    vec![
                        #gen_helpers
                    ]
                }
            }
        };
        gen.into()
    } else {
        Diagnostic::spanned(
            ast.span().unwrap(),
            proc_macro::Level::Error,
            "#[derive(MaterialData)] can only be used on structs",
        )
        .emit();
        TokenStream::new()
    }
}
