macro_rules! __solverforge_list_owner_public_methods {
    ($entity_collections:ident, $owner_public_methods:ident) => {
    let $owner_public_methods: Vec<_> = $entity_collections
        .iter()
        .map(|(descriptor_index, field_ident, entity_type)| {
            let field_name = field_ident.to_string();
            let list_trait =
                quote! { <#entity_type as ::solverforge::__internal::ListVariableEntity<Self>> };
            let owner_guard = quote! {
                if !#list_trait::HAS_LIST_VARIABLE {
                    panic!(
                        "`{}` is not a planning list owner on this solution",
                        stringify!(#field_ident)
                    );
                }
            };
            let list_len_ident = format_ident!("__solverforge_list_len_{}", field_name);
            let list_remove_ident = format_ident!("__solverforge_list_remove_{}", field_name);
            let list_insert_ident = format_ident!("__solverforge_list_insert_{}", field_name);
            let list_get_ident = format_ident!("__solverforge_list_get_{}", field_name);
            let list_set_ident = format_ident!("__solverforge_list_set_{}", field_name);
            let list_reverse_ident = format_ident!("__solverforge_list_reverse_{}", field_name);
            let sublist_remove_ident = format_ident!("__solverforge_sublist_remove_{}", field_name);
            let sublist_insert_ident = format_ident!("__solverforge_sublist_insert_{}", field_name);
            let ruin_remove_ident = format_ident!("__solverforge_ruin_remove_{}", field_name);
            let ruin_insert_ident = format_ident!("__solverforge_ruin_insert_{}", field_name);
            let list_remove_for_construction_ident =
                format_ident!("__solverforge_list_remove_for_construction_{}", field_name);
            let index_to_element_ident =
                format_ident!("__solverforge_index_to_element_{}", field_name);
            let element_count_ident = format_ident!("__solverforge_element_count_{}", field_name);
            let assigned_elements_ident =
                format_ident!("__solverforge_assigned_elements_{}", field_name);
            let n_entities_ident = format_ident!("__solverforge_n_entities_{}", field_name);
            let assign_element_ident = format_ident!("__solverforge_assign_element_{}", field_name);

            let owner_list_len_method = format_ident!("{}_list_len", field_name);
            let owner_list_len_static_method = format_ident!("{}_list_len_static", field_name);
            let owner_list_remove_method = format_ident!("{}_list_remove", field_name);
            let owner_list_insert_method = format_ident!("{}_list_insert", field_name);
            let owner_list_get_method = format_ident!("{}_list_get", field_name);
            let owner_list_set_method = format_ident!("{}_list_set", field_name);
            let owner_list_reverse_method = format_ident!("{}_list_reverse", field_name);
            let owner_sublist_remove_method = format_ident!("{}_sublist_remove", field_name);
            let owner_sublist_insert_method = format_ident!("{}_sublist_insert", field_name);
            let owner_ruin_remove_method = format_ident!("{}_ruin_remove", field_name);
            let owner_ruin_insert_method = format_ident!("{}_ruin_insert", field_name);
            let owner_list_remove_for_construction_method =
                format_ident!("{}_list_remove_for_construction", field_name);
            let owner_index_to_element_method =
                format_ident!("{}_index_to_element_static", field_name);
            let owner_descriptor_index_method =
                format_ident!("{}_list_variable_descriptor_index", field_name);
            let owner_element_count_method = format_ident!("{}_element_count", field_name);
            let owner_assigned_elements_method = format_ident!("{}_assigned_elements", field_name);
            let owner_n_entities_method = format_ident!("{}_n_entities", field_name);
            let owner_assign_element_method = format_ident!("{}_assign_element", field_name);

            let descriptor_index_lit = syn::LitInt::new(
                &descriptor_index.to_string(),
                proc_macro2::Span::call_site(),
            );

            quote! {
                #[inline]
                pub fn #owner_list_len_method(&self, entity_idx: usize) -> usize {
                    #owner_guard
                    Self::#list_len_ident(self, entity_idx)
                }

                #[inline]
                pub fn #owner_list_len_static_method(s: &Self, entity_idx: usize) -> usize {
                    #owner_guard
                    Self::#list_len_ident(s, entity_idx)
                }

                #[inline]
                pub fn #owner_list_remove_method(
                    s: &mut Self,
                    entity_idx: usize,
                    pos: usize,
                ) -> ::core::option::Option<usize> {
                    #owner_guard
                    Self::#list_remove_ident(s, entity_idx, pos)
                }

                #[inline]
                pub fn #owner_list_insert_method(
                    s: &mut Self,
                    entity_idx: usize,
                    pos: usize,
                    val: usize,
                ) {
                    #owner_guard
                    Self::#list_insert_ident(s, entity_idx, pos, val)
                }

                #[inline]
                pub fn #owner_list_get_method(
                    s: &Self,
                    entity_idx: usize,
                    pos: usize,
                ) -> ::core::option::Option<usize> {
                    #owner_guard
                    Self::#list_get_ident(s, entity_idx, pos)
                }

                #[inline]
                pub fn #owner_list_set_method(
                    s: &mut Self,
                    entity_idx: usize,
                    pos: usize,
                    val: usize,
                ) {
                    #owner_guard
                    Self::#list_set_ident(s, entity_idx, pos, val)
                }

                #[inline]
                pub fn #owner_list_reverse_method(
                    s: &mut Self,
                    entity_idx: usize,
                    start: usize,
                    end: usize,
                ) {
                    #owner_guard
                    Self::#list_reverse_ident(s, entity_idx, start, end)
                }

                #[inline]
                pub fn #owner_sublist_remove_method(
                    s: &mut Self,
                    entity_idx: usize,
                    start: usize,
                    end: usize,
                ) -> Vec<usize> {
                    #owner_guard
                    Self::#sublist_remove_ident(s, entity_idx, start, end)
                }

                #[inline]
                pub fn #owner_sublist_insert_method(
                    s: &mut Self,
                    entity_idx: usize,
                    pos: usize,
                    items: Vec<usize>,
                ) {
                    #owner_guard
                    Self::#sublist_insert_ident(s, entity_idx, pos, items)
                }

                #[inline]
                pub fn #owner_ruin_remove_method(
                    s: &mut Self,
                    entity_idx: usize,
                    pos: usize,
                ) -> usize {
                    #owner_guard
                    Self::#ruin_remove_ident(s, entity_idx, pos)
                }

                #[inline]
                pub fn #owner_ruin_insert_method(
                    s: &mut Self,
                    entity_idx: usize,
                    pos: usize,
                    val: usize,
                ) {
                    #owner_guard
                    Self::#ruin_insert_ident(s, entity_idx, pos, val)
                }

                #[inline]
                pub fn #owner_list_remove_for_construction_method(
                    s: &mut Self,
                    entity_idx: usize,
                    pos: usize,
                ) -> usize {
                    #owner_guard
                    Self::#list_remove_for_construction_ident(s, entity_idx, pos)
                }

                #[inline]
                pub fn #owner_index_to_element_method(s: &Self, idx: usize) -> usize {
                    #owner_guard
                    Self::#index_to_element_ident(s, idx)
                }

                #[inline]
                pub fn #owner_descriptor_index_method() -> usize {
                    #owner_guard
                    #descriptor_index_lit
                }

                #[inline]
                pub fn #owner_element_count_method(s: &Self) -> usize {
                    #owner_guard
                    Self::#element_count_ident(s)
                }

                #[inline]
                pub fn #owner_assigned_elements_method(s: &Self) -> Vec<usize> {
                    #owner_guard
                    Self::#assigned_elements_ident(s)
                }

                #[inline]
                pub fn #owner_n_entities_method(s: &Self) -> usize {
                    #owner_guard
                    Self::#n_entities_ident(s)
                }

                #[inline]
                pub fn #owner_assign_element_method(
                    s: &mut Self,
                    entity_idx: usize,
                    elem: usize,
                ) {
                    #owner_guard
                    Self::#assign_element_ident(s, entity_idx, elem)
                }
            }
        })
        .collect();

    };
}
