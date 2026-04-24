use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::attr_parse::has_attribute;

use super::type_helpers::extract_collection_inner_type;

include!("list_operations/setup.rs");
include!("list_operations/owner_public_methods.rs");
include!("list_operations/single_owner_branches.rs");
include!("list_operations/quote.rs");

pub(super) fn generate_list_operations(
    fields: &syn::punctuated::Punctuated<syn::Field, syn::token::Comma>,
) -> TokenStream {
    __solverforge_list_setup!(
        fields,
        entity_collections,
        source_len_arms,
        source_element_arms,
        owner_helpers
    );
    __solverforge_list_owner_public_methods!(entity_collections, owner_public_methods);
    __solverforge_list_single_owner_branches!(
        entity_collections,
        list_owner_count_terms,
        single_owner_list_len_branches,
        single_owner_list_remove_branches,
        single_owner_list_insert_branches,
        single_owner_list_get_branches,
        single_owner_list_set_branches,
        single_owner_list_reverse_branches,
        single_owner_sublist_remove_branches,
        single_owner_sublist_insert_branches,
        single_owner_ruin_remove_branches,
        single_owner_ruin_insert_branches,
        single_owner_remove_for_construction_branches,
        single_owner_index_to_element_branches,
        single_owner_descriptor_index_branches,
        single_owner_element_count_branches,
        single_owner_assigned_elements_branches,
        single_owner_n_entities_branches,
        single_owner_assign_element_branches,
        total_list_entities_terms,
        total_list_elements_terms
    );
    __solverforge_list_quote!(
        owner_helpers,
        list_owner_count_terms,
        owner_public_methods,
        single_owner_list_len_branches,
        single_owner_list_remove_branches,
        single_owner_list_insert_branches,
        single_owner_list_get_branches,
        single_owner_list_set_branches,
        single_owner_list_reverse_branches,
        single_owner_sublist_remove_branches,
        single_owner_sublist_insert_branches,
        single_owner_ruin_remove_branches,
        single_owner_ruin_insert_branches,
        single_owner_remove_for_construction_branches,
        single_owner_index_to_element_branches,
        single_owner_descriptor_index_branches,
        single_owner_element_count_branches,
        single_owner_assigned_elements_branches,
        single_owner_n_entities_branches,
        single_owner_assign_element_branches,
        total_list_entities_terms,
        total_list_elements_terms
    )
}
