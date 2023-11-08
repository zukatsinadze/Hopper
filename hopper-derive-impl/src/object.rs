use crate::field;
use crate::my_quote;
use crate::serde;

use proc_macro2::TokenStream;
use syn::{punctuated::Punctuated, Token};

/// Add implmentation of object's fuzzing trait for struct types
pub fn object_trait_for_struct(
    name: &syn::Ident,
    generics: &syn::Generics,
    attrs: &[syn::Attribute],
    fields: &syn::Fields,
) -> TokenStream {
    let crate_path = super::get_crate_path();
    let serde = serde::serde_trait_for_struct(name, generics, attrs, fields);
    let object_serde = serde::object_serde_trait_for_struct(name, generics, attrs, fields);
    let (fields, named, unit) = field::convert_fields(fields);
    let gen_body = field::struct_object_gen_body(&fields, unit, named);
    let mutate_body = field::struct_object_mutate_body(&fields, unit, false);
    let det_mutate_body = field::struct_object_mutate_body(&fields, unit, true);
    let mutate_op_body = field::struct_object_mutate_op_body(&fields, unit);
    let ptr_body = field::struct_object_ptr_body(&fields, unit);
    let layout_body = field::struct_object_layout_body(&fields, unit);
    // let is_opaque = fields.iter().any(|f| f.is_opaque());
    let is_opaque = field::struct_object_opaque_body(&fields);
  
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let add_types = fields.iter().map(|f| {
        let f_ty = f.ty;
        my_quote!(gadgets.add_type::<#f_ty>();)
    });
    let field_ty = fields.iter().map(|f| {
        let f_ty = f.ty;
        let f_name = f.ident.to_string();
        my_quote!(
            ret.insert(#f_name.to_string(), std::any::type_name::<#f_ty>().to_owned());
        )
    }); 

    my_quote! {
        impl #impl_generics #crate_path::ObjFuzzable for #name #ty_generics #where_clause {
        }
        impl #impl_generics #crate_path::ObjGenerate for #name #ty_generics #where_clause {
            fn generate_new(state: &mut #crate_path::ObjectState) -> eyre::Result<Self> {
                let val = #name #gen_body;
                Ok(val)
            }
        }
        impl #impl_generics #crate_path::ObjMutate for #name #ty_generics #where_clause {
            fn det_mutate(&mut self, state: &mut #crate_path::ObjectState) -> eyre::Result<#crate_path::MutateOperator> {
                #det_mutate_body
            }
            fn mutate(&mut self, state: &mut #crate_path::ObjectState) -> eyre::Result<#crate_path::MutateOperator> {
                #mutate_body
            }
            fn mutate_by_op(&mut self, state: &mut #crate_path::ObjectState,
                keys: &[#crate_path::FieldKey], op: &#crate_path::MutateOperation) -> eyre::Result<()> {
                #mutate_op_body
            }
        }
        impl #impl_generics #crate_path::ObjValue for #name #ty_generics #where_clause {
            fn get_layout(&self, fold_ptr: bool) -> #crate_path::ObjectLayout {
                let mut layout = #crate_path::ObjectLayout::root(self.type_name(), self as *const Self as *mut u8);
                #layout_body
                layout
            }
            fn get_ptr_by_keys(&self, keys: &[#crate_path::FieldKey]) -> eyre::Result<*mut u8> {
                #ptr_body
            }
        }
        impl #impl_generics #crate_path::ObjType for #name #ty_generics #where_clause {
            fn is_opaque() -> bool {
                #is_opaque
            }

            fn add_fields_to_gadgets(gadgets: &mut #crate_path::ProgramGadgets) {
                #(#add_types)*
            }

            fn get_fields_ty() -> std::collections::HashMap<String, String> {
                let mut ret = std::collections::HashMap::default();
                #(#field_ty)*
                ret
            }
        }
        #serde
        #object_serde
    }
}

pub fn object_trait_for_union(
    name: &syn::Ident,
    generics: &syn::Generics,
    attrs: &[syn::Attribute],
    fields: &syn::FieldsNamed,
) -> TokenStream {
    let crate_path = super::get_crate_path();
    let serde = serde::serde_trait_for_union(name, generics, attrs, fields);
    let object_serde = serde::object_serde_trait_for_union(name, generics, attrs, fields);
    let fields = field::convert_field_list(Some(&fields.named), true);
    let gen_body = field::union_object_gen_body(&fields);
    let mutate_body = field::union_object_mutate_body(&fields);
    let mutate_op_body = field::struct_object_mutate_op_body(&fields, false);
    let mutate_union_use = field::union_object_use_member(&fields);
    let ptr_body = field::struct_object_ptr_body(&fields, false);
    let add_types = fields.iter().map(|f| {
        let f_ty = f.ty;
        my_quote!(gadgets.add_type::<#f_ty>();)
    });
    let field_ty = fields.iter().map(|f| {
        let f_ty = f.ty;
        let f_name = f.ident.to_string();
        my_quote!(
            ret.insert(#f_name.to_string(), std::any::type_name::<#f_ty>().to_owned());
        )
    }); 
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    my_quote! {
        impl #impl_generics #crate_path::ObjFuzzable for #name #ty_generics #where_clause {
        }
        impl #impl_generics #crate_path::ObjGenerate for #name #ty_generics #where_clause {
            fn generate_new(state: &mut #crate_path::ObjectState) -> eyre::Result<Self> {
                state.is_union = true;
                let val = #gen_body;
                Ok(val)
            }
        }
        impl #impl_generics #crate_path::ObjMutate for #name #ty_generics #where_clause {
            fn mutate(&mut self, state: &mut #crate_path::ObjectState) -> eyre::Result<#crate_path::MutateOperator> {
                #mutate_body
            }
            fn mutate_by_op(&mut self, state: &mut #crate_path::ObjectState,
                keys: &[#crate_path::FieldKey], op: &#crate_path::MutateOperation) -> eyre::Result<()> {
                use hopper::ObjGenerate;
                unsafe {
                    match op {
                        #crate_path::MutateOperation::UnionUse{ member, .. } => {
                            #mutate_union_use
                        }
                        _ => {
                            #mutate_op_body
                        }
                    }
                    Ok(())
                }
            }
        }
        impl #impl_generics #crate_path::ObjValue for #name #ty_generics #where_clause {
            fn get_layout(&self, _fold_ptr: bool) -> #crate_path::ObjectLayout {
                let mut layout = #crate_path::ObjectLayout::root(
                    std::any::type_name::<Self>(),
                    self as *const Self as *mut u8,
                );
                layout.is_union = true;
                layout
            }

            fn get_ptr_by_keys(&self, keys: &[#crate_path::FieldKey]) -> eyre::Result<*mut u8> {
                unsafe {
                    #ptr_body
                }
            }
        }
        impl #impl_generics #crate_path::ObjType for #name #ty_generics #where_clause {
            fn add_fields_to_gadgets(gadgets: &mut #crate_path::ProgramGadgets) {
                #(#add_types)*
            }

            fn get_fields_ty() -> std::collections::HashMap<String, String> {
                let mut ret = std::collections::HashMap::default();
                #(#field_ty)*
                ret
            }
        }
        #serde
        #object_serde
    }
}

/// Add implmentation of object's fuzzing trait for enum types
/// TODO: implement variants
pub fn object_trait_for_enum(
    _name: &syn::Ident,
    _generics: &syn::Generics,
    _attrs: &[syn::Attribute],
    variants: &Punctuated<syn::Variant, Token![,]>,
) -> TokenStream {
    if variants.is_empty() {
        panic!("#[derive(Hopper)] cannot be implemented for enums with zero variants");
    }
    let impls = variants.iter().map(|v| {
        if v.discriminant.is_some() {
            panic!("#[derive(Hopper)] cannot be implemented for enums with discriminants");
        }
        // qual = my_quote!(::#variant),
        //object_trait_for_struct(name, generics, attrs, &v.fields, Some(&v.ident))
        my_quote!()
    });
    my_quote!(#(#impls)*)
}
