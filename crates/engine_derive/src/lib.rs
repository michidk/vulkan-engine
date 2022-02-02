use quote::{quote, quote_spanned};
use syn::{parse_macro_input, Data, DeriveInput};

#[proc_macro_derive(InternalComponent, attributes(custom_inspector, inspector))]
pub fn derive_component_internal(target: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let target = parse_macro_input!(target as DeriveInput);
    let data = if let Data::Struct(d) = target.data {
        d
    } else {
        return error(target.ident.span(), "must be a struct");
    };
    let name = target.ident;
    let name_str = name.to_string();

    let impl_name = quote! {
        impl crate::scene::component::ComponentName for #name {
            fn component_name(&self) -> &'static str {
                Self::static_component_name()
            }
            fn static_component_name() -> &'static str where Self: Sized {
                #name_str
            }
        }
    };

    let custom_inspector = target
        .attrs
        .iter()
        .any(|attr| attr.path.is_ident("custom_inspector"));

    let impl_inspector = if custom_inspector {
        quote! {}
    } else {
        let mut impl_inspector = proc_macro2::TokenStream::new();
        for field in data.fields {
            let field_name = field.ident.unwrap();

            if let Some(inspector_attr) = field
                .attrs
                .iter()
                .find(|attr| attr.path.is_ident("inspector"))
            {
                let widget = &inspector_attr.tokens;

                impl_inspector.extend(quote! {
                    #widget.render_value(ui, &self.#field_name);
                });
            }
        }

        quote! {
            impl crate::scene::component::ComponentInspector for #name {
                fn render_inspector(&self, ui: &mut egui::Ui) {
                    use crate::scene::component::InspectableValueRead;
                    use crate::scene::component::InspectableValue;
                    use crate::scene::component::InspectableRendererRead;
                    use crate::scene::component::InspectableRenderer;

                    #impl_inspector
                }
            }
        }
    };

    quote! {
        #impl_name

        #impl_inspector
    }
    .into()
}

#[proc_macro_derive(Component, attributes(custom_inspector, inspector))]
pub fn derive_component(target: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let target = parse_macro_input!(target as DeriveInput);
    let data = if let Data::Struct(d) = target.data {
        d
    } else {
        return error(target.ident.span(), "must be a struct");
    };
    let name = target.ident;
    let name_str = name.to_string();

    let impl_name = quote! {
        impl vulkan_engine::scene::component::ComponentName for #name {
            fn component_name(&self) -> &'static str {
                Self::static_component_name()
            }
            fn static_component_name() -> &'static str where Self: Sized {
                #name_str
            }
        }
    };

    let custom_inspector = target
        .attrs
        .iter()
        .any(|attr| attr.path.is_ident("custom_inspector"));

    let impl_inspector = if custom_inspector {
        quote! {}
    } else {
        let mut impl_inspector = proc_macro2::TokenStream::new();
        for field in data.fields {
            let field_name = field.ident.unwrap();

            if let Some(inspector_attr) = field
                .attrs
                .iter()
                .find(|attr| attr.path.is_ident("inspector"))
            {
                let widget = &inspector_attr.tokens;

                impl_inspector.extend(quote! {
                    #widget.render_value(ui, &self.#field_name);
                });
            }
        }

        quote! {
            impl vulkan_engine::scene::component::ComponentInspector for #name {
                fn render_inspector(&self, ui: &mut egui::Ui) {
                    use vulkan_engine::scene::component::InspectableValueRead;
                    use vulkan_engine::scene::component::InspectableValue;
                    use vulkan_engine::scene::component::InspectableRendererRead;
                    use vulkan_engine::scene::component::InspectableRenderer;

                    #impl_inspector
                }
            }
        }
    };

    quote! {
        #impl_name

        #impl_inspector
    }
    .into()
}

fn error(span: proc_macro2::Span, error: &str) -> proc_macro::TokenStream {
    quote_spanned! {
        span =>
        compile_error!(#error);
    }
    .into()
}
