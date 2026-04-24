macro_rules! __solverforge_list_single_owner_branches {
    ($entity_collections:ident, $list_owner_count_terms:ident, $single_owner_list_len_branches:ident, $single_owner_list_remove_branches:ident, $single_owner_list_insert_branches:ident, $single_owner_list_get_branches:ident, $single_owner_list_set_branches:ident, $single_owner_list_reverse_branches:ident, $single_owner_sublist_remove_branches:ident, $single_owner_sublist_insert_branches:ident, $single_owner_ruin_remove_branches:ident, $single_owner_ruin_insert_branches:ident, $single_owner_remove_for_construction_branches:ident, $single_owner_index_to_element_branches:ident, $single_owner_descriptor_index_branches:ident, $single_owner_element_count_branches:ident, $single_owner_assigned_elements_branches:ident, $single_owner_n_entities_branches:ident, $single_owner_assign_element_branches:ident, $total_list_entities_terms:ident, $total_list_elements_terms:ident) => {
    let $list_owner_count_terms: Vec<_> = $entity_collections
        .iter()
        .map(|(_, _, entity_type)| quote! { #entity_type::__SOLVERFORGE_LIST_VARIABLE_COUNT })
        .collect();

    let $single_owner_list_len_branches: Vec<_> = $entity_collections
        .iter()
        .map(|(_, field_ident, entity_type)| {
            let field_name = field_ident.to_string();
            let list_trait =
                quote! { <#entity_type as ::solverforge::__internal::ListVariableEntity<Self>> };
            let list_len_ident = format_ident!("__solverforge_list_len_{}", field_name);
            quote! {
                if #list_trait::HAS_LIST_VARIABLE {
                    return Self::#list_len_ident(s, entity_idx);
                }
            }
        })
        .collect();

    let $single_owner_list_remove_branches: Vec<_> = $entity_collections
        .iter()
        .map(|(_, field_ident, entity_type)| {
            let field_name = field_ident.to_string();
            let list_trait =
                quote! { <#entity_type as ::solverforge::__internal::ListVariableEntity<Self>> };
            let list_remove_ident = format_ident!("__solverforge_list_remove_{}", field_name);
            quote! {
                if #list_trait::HAS_LIST_VARIABLE {
                    return Self::#list_remove_ident(s, entity_idx, pos);
                }
            }
        })
        .collect();

    let $single_owner_list_insert_branches: Vec<_> = $entity_collections
        .iter()
        .map(|(_, field_ident, entity_type)| {
            let field_name = field_ident.to_string();
            let list_trait =
                quote! { <#entity_type as ::solverforge::__internal::ListVariableEntity<Self>> };
            let list_insert_ident = format_ident!("__solverforge_list_insert_{}", field_name);
            quote! {
                if #list_trait::HAS_LIST_VARIABLE {
                    Self::#list_insert_ident(s, entity_idx, pos, val);
                    return;
                }
            }
        })
        .collect();

    let $single_owner_list_get_branches: Vec<_> = $entity_collections
        .iter()
        .map(|(_, field_ident, entity_type)| {
            let field_name = field_ident.to_string();
            let list_trait =
                quote! { <#entity_type as ::solverforge::__internal::ListVariableEntity<Self>> };
            let list_get_ident = format_ident!("__solverforge_list_get_{}", field_name);
            quote! {
                if #list_trait::HAS_LIST_VARIABLE {
                    return Self::#list_get_ident(s, entity_idx, pos);
                }
            }
        })
        .collect();

    let $single_owner_list_set_branches: Vec<_> = $entity_collections
        .iter()
        .map(|(_, field_ident, entity_type)| {
            let field_name = field_ident.to_string();
            let list_trait =
                quote! { <#entity_type as ::solverforge::__internal::ListVariableEntity<Self>> };
            let list_set_ident = format_ident!("__solverforge_list_set_{}", field_name);
            quote! {
                if #list_trait::HAS_LIST_VARIABLE {
                    Self::#list_set_ident(s, entity_idx, pos, val);
                    return;
                }
            }
        })
        .collect();

    let $single_owner_list_reverse_branches: Vec<_> = $entity_collections
        .iter()
        .map(|(_, field_ident, entity_type)| {
            let field_name = field_ident.to_string();
            let list_trait =
                quote! { <#entity_type as ::solverforge::__internal::ListVariableEntity<Self>> };
            let list_reverse_ident = format_ident!("__solverforge_list_reverse_{}", field_name);
            quote! {
                if #list_trait::HAS_LIST_VARIABLE {
                    Self::#list_reverse_ident(s, entity_idx, start, end);
                    return;
                }
            }
        })
        .collect();

    let $single_owner_sublist_remove_branches: Vec<_> = $entity_collections
        .iter()
        .map(|(_, field_ident, entity_type)| {
            let field_name = field_ident.to_string();
            let list_trait =
                quote! { <#entity_type as ::solverforge::__internal::ListVariableEntity<Self>> };
            let sublist_remove_ident = format_ident!("__solverforge_sublist_remove_{}", field_name);
            quote! {
                if #list_trait::HAS_LIST_VARIABLE {
                    return Self::#sublist_remove_ident(s, entity_idx, start, end);
                }
            }
        })
        .collect();

    let $single_owner_sublist_insert_branches: Vec<_> = $entity_collections
        .iter()
        .map(|(_, field_ident, entity_type)| {
            let field_name = field_ident.to_string();
            let list_trait =
                quote! { <#entity_type as ::solverforge::__internal::ListVariableEntity<Self>> };
            let sublist_insert_ident = format_ident!("__solverforge_sublist_insert_{}", field_name);
            quote! {
                if #list_trait::HAS_LIST_VARIABLE {
                    Self::#sublist_insert_ident(s, entity_idx, pos, items);
                    return;
                }
            }
        })
        .collect();

    let $single_owner_ruin_remove_branches: Vec<_> = $entity_collections
        .iter()
        .map(|(_, field_ident, entity_type)| {
            let field_name = field_ident.to_string();
            let list_trait =
                quote! { <#entity_type as ::solverforge::__internal::ListVariableEntity<Self>> };
            let ruin_remove_ident = format_ident!("__solverforge_ruin_remove_{}", field_name);
            quote! {
                if #list_trait::HAS_LIST_VARIABLE {
                    return Self::#ruin_remove_ident(s, entity_idx, pos);
                }
            }
        })
        .collect();

    let $single_owner_ruin_insert_branches: Vec<_> = $entity_collections
        .iter()
        .map(|(_, field_ident, entity_type)| {
            let field_name = field_ident.to_string();
            let list_trait =
                quote! { <#entity_type as ::solverforge::__internal::ListVariableEntity<Self>> };
            let ruin_insert_ident = format_ident!("__solverforge_ruin_insert_{}", field_name);
            quote! {
                if #list_trait::HAS_LIST_VARIABLE {
                    Self::#ruin_insert_ident(s, entity_idx, pos, val);
                    return;
                }
            }
        })
        .collect();

    let $single_owner_remove_for_construction_branches: Vec<_> = $entity_collections
        .iter()
        .map(|(_, field_ident, entity_type)| {
            let field_name = field_ident.to_string();
            let list_trait =
                quote! { <#entity_type as ::solverforge::__internal::ListVariableEntity<Self>> };
            let list_remove_for_construction_ident =
                format_ident!("__solverforge_list_remove_for_construction_{}", field_name);
            quote! {
                if #list_trait::HAS_LIST_VARIABLE {
                    return Self::#list_remove_for_construction_ident(s, entity_idx, pos);
                }
            }
        })
        .collect();

    let $single_owner_index_to_element_branches: Vec<_> = $entity_collections
        .iter()
        .map(|(_, field_ident, entity_type)| {
            let field_name = field_ident.to_string();
            let list_trait =
                quote! { <#entity_type as ::solverforge::__internal::ListVariableEntity<Self>> };
            let index_to_element_ident =
                format_ident!("__solverforge_index_to_element_{}", field_name);
            quote! {
                if #list_trait::HAS_LIST_VARIABLE {
                    return Self::#index_to_element_ident(s, idx);
                }
            }
        })
        .collect();

    let $single_owner_descriptor_index_branches: Vec<_> = $entity_collections
        .iter()
        .map(|(descriptor_index, _, entity_type)| {
            let descriptor_index_lit = syn::LitInt::new(
                &descriptor_index.to_string(),
                proc_macro2::Span::call_site(),
            );
            let list_trait =
                quote! { <#entity_type as ::solverforge::__internal::ListVariableEntity<Self>> };
            quote! {
                if #list_trait::HAS_LIST_VARIABLE {
                    return #descriptor_index_lit;
                }
            }
        })
        .collect();

    let $single_owner_element_count_branches: Vec<_> = $entity_collections
        .iter()
        .map(|(_, field_ident, entity_type)| {
            let field_name = field_ident.to_string();
            let list_trait =
                quote! { <#entity_type as ::solverforge::__internal::ListVariableEntity<Self>> };
            let element_count_ident = format_ident!("__solverforge_element_count_{}", field_name);
            quote! {
                if #list_trait::HAS_LIST_VARIABLE {
                    return Self::#element_count_ident(s);
                }
            }
        })
        .collect();

    let $single_owner_assigned_elements_branches: Vec<_> = $entity_collections
        .iter()
        .map(|(_, field_ident, entity_type)| {
            let field_name = field_ident.to_string();
            let list_trait =
                quote! { <#entity_type as ::solverforge::__internal::ListVariableEntity<Self>> };
            let assigned_elements_ident =
                format_ident!("__solverforge_assigned_elements_{}", field_name);
            quote! {
                if #list_trait::HAS_LIST_VARIABLE {
                    return Self::#assigned_elements_ident(s);
                }
            }
        })
        .collect();

    let $single_owner_n_entities_branches: Vec<_> = $entity_collections
        .iter()
        .map(|(_, field_ident, entity_type)| {
            let field_name = field_ident.to_string();
            let list_trait =
                quote! { <#entity_type as ::solverforge::__internal::ListVariableEntity<Self>> };
            let n_entities_ident = format_ident!("__solverforge_n_entities_{}", field_name);
            quote! {
                if #list_trait::HAS_LIST_VARIABLE {
                    return Self::#n_entities_ident(s);
                }
            }
        })
        .collect();

    let $single_owner_assign_element_branches: Vec<_> = $entity_collections
        .iter()
        .map(|(_, field_ident, entity_type)| {
            let field_name = field_ident.to_string();
            let list_trait =
                quote! { <#entity_type as ::solverforge::__internal::ListVariableEntity<Self>> };
            let assign_element_ident = format_ident!("__solverforge_assign_element_{}", field_name);
            quote! {
                if #list_trait::HAS_LIST_VARIABLE {
                    Self::#assign_element_ident(s, entity_idx, elem);
                    return;
                }
            }
        })
        .collect();

    let $total_list_entities_terms: Vec<_> = $entity_collections
        .iter()
        .map(|(_, field_ident, entity_type)| {
            let field_name = field_ident.to_string();
            let list_trait =
                quote! { <#entity_type as ::solverforge::__internal::ListVariableEntity<Self>> };
            let n_entities_ident = format_ident!("__solverforge_n_entities_{}", field_name);
            quote! {
                if #list_trait::HAS_LIST_VARIABLE {
                    Self::#n_entities_ident(s)
                } else {
                    0
                }
            }
        })
        .collect();

    let $total_list_elements_terms: Vec<_> = $entity_collections
        .iter()
        .map(|(_, field_ident, entity_type)| {
            let field_name = field_ident.to_string();
            let list_trait =
                quote! { <#entity_type as ::solverforge::__internal::ListVariableEntity<Self>> };
            let element_count_ident = format_ident!("__solverforge_element_count_{}", field_name);
            quote! {
                if #list_trait::HAS_LIST_VARIABLE {
                    Self::#element_count_ident(s)
                } else {
                    0
                }
            }
        })
        .collect();
    };
}
