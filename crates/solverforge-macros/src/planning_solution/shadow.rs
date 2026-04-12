use proc_macro2::TokenStream;
use quote::quote;
use syn::Error;
use syn::Ident;

use super::config::ShadowConfig;
use super::list_runtime::{
    find_list_owner_config, find_list_runtime_config, shadow_updates_requested,
    ListElementCollectionKind,
};

pub(super) fn generate_shadow_support(
    config: &ShadowConfig,
    fields: &syn::punctuated::Punctuated<syn::Field, syn::token::Comma>,
    solution_name: &Ident,
) -> Result<TokenStream, Error> {
    if !shadow_updates_requested(config) {
        return Ok(quote! {
            impl ::solverforge::__internal::ShadowVariableSupport for #solution_name {
                #[inline]
                fn update_entity_shadows(&mut self, _entity_idx: usize) {}
            }
        });
    }

    let Some(list_owner) = find_list_owner_config(config, fields)? else {
        return Err(Error::new(
            proc_macro2::Span::call_site(),
            "#[shadow_variable_updates(...)] requires `list_owner = \"entity_collection_field\"` when shadow updates are configured",
        ));
    };

    let Some(runtime_config) = find_list_runtime_config(fields)? else {
        return Err(Error::new(
            proc_macro2::Span::call_site(),
            format!(
                "planning solution with list owner `{}` requires a `#[planning_entity_collection]` or `#[problem_fact_collection]` field named `{}`",
                list_owner.field_ident,
                list_owner.field_ident,
            ),
        ));
    };
    if runtime_config.list_owner.field_ident != list_owner.field_ident {
        return Err(Error::new(
            proc_macro2::Span::call_site(),
            format!(
                "#[shadow_variable_updates(list_owner = \"{}\")] does not match the inferred list owner `{}`",
                list_owner.field_ident,
                runtime_config.list_owner.field_ident,
            ),
        ));
    }
    if runtime_config.element_collection.kind == ListElementCollectionKind::LegacyListCollection {
        return Err(Error::new(
            proc_macro2::Span::call_site(),
            format!(
                "planning solution with list owner `{}` requires a matching `#[planning_entity_collection]` or `#[problem_fact_collection]` field for shadow updates",
                list_owner.field_ident,
            ),
        ));
    }

    let list_owner_ident = list_owner.field_ident;
    let element_collection_ident = runtime_config.element_collection.field_ident;
    let list_owner_type = list_owner.entity_type;
    let list_trait =
        quote! { <#list_owner_type as ::solverforge::__internal::ListVariableEntity<Self>> };

    let inverse_update = config.inverse_field.as_ref().map(|field| {
        let field_ident = Ident::new(field, proc_macro2::Span::call_site());
        quote! {
            for &element_idx in &element_indices {
                self.#element_collection_ident[element_idx].#field_ident = Some(entity_idx);
            }
        }
    });

    let previous_update = config.previous_field.as_ref().map(|field| {
        let field_ident = Ident::new(field, proc_macro2::Span::call_site());
        quote! {
            let mut prev_idx: Option<usize> = None;
            for &element_idx in &element_indices {
                self.#element_collection_ident[element_idx].#field_ident = prev_idx;
                prev_idx = Some(element_idx);
            }
        }
    });

    let next_update = config.next_field.as_ref().map(|field| {
        let field_ident = Ident::new(field, proc_macro2::Span::call_site());
        quote! {
            let len = element_indices.len();
            for (i, &element_idx) in element_indices.iter().enumerate() {
                let next_idx = if i + 1 < len { Some(element_indices[i + 1]) } else { None };
                self.#element_collection_ident[element_idx].#field_ident = next_idx;
            }
        }
    });

    let cascading_update = config.cascading_listener.as_ref().map(|method| {
        let method_ident = Ident::new(method, proc_macro2::Span::call_site());
        quote! {
            for &element_idx in &element_indices {
                self.#method_ident(element_idx);
            }
        }
    });

    let post_update = config.post_update_listener.as_ref().map(|method| {
        let method_ident = Ident::new(method, proc_macro2::Span::call_site());
        quote! {
            self.#method_ident(entity_idx);
        }
    });

    let aggregate_updates: Vec<_> = config
        .entity_aggregates
        .iter()
        .filter_map(|spec| {
            let parts: Vec<&str> = spec.split(':').collect();
            if parts.len() != 3 {
                return None;
            }
            let target_field = Ident::new(parts[0], proc_macro2::Span::call_site());
            let aggregation = parts[1];
            let source_field = Ident::new(parts[2], proc_macro2::Span::call_site());

            match aggregation {
                "sum" => Some(quote! {
                    self.#list_owner_ident[entity_idx].#target_field = element_indices
                        .iter()
                        .map(|&idx| self.#element_collection_ident[idx].#source_field)
                        .sum();
                }),
                _ => None,
            }
        })
        .collect();

    let compute_updates: Vec<_> = config
        .entity_computes
        .iter()
        .filter_map(|spec| {
            let parts: Vec<&str> = spec.split(':').collect();
            if parts.len() != 2 {
                return None;
            }
            let target_field = Ident::new(parts[0], proc_macro2::Span::call_site());
            let method_name = Ident::new(parts[1], proc_macro2::Span::call_site());

            Some(quote! {
                self.#list_owner_ident[entity_idx].#target_field = self.#method_name(entity_idx);
            })
        })
        .collect();

    Ok(quote! {
        impl ::solverforge::__internal::ShadowVariableSupport for #solution_name {
            #[inline]
            fn update_entity_shadows(&mut self, entity_idx: usize) {
                let element_indices: Vec<usize> =
                    #list_trait::list_field(&self.#list_owner_ident[entity_idx]).to_vec();

                #inverse_update
                #previous_update
                #next_update
                #cascading_update
                #(#aggregate_updates)*
                #(#compute_updates)*
                #post_update
            }
        }
    })
}
