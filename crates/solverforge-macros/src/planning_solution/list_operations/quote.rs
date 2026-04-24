macro_rules! __solverforge_list_quote {
    ($owner_helpers:ident, $list_owner_count_terms:ident, $owner_public_methods:ident, $single_owner_list_len_branches:ident, $single_owner_list_remove_branches:ident, $single_owner_list_insert_branches:ident, $single_owner_list_get_branches:ident, $single_owner_list_set_branches:ident, $single_owner_list_reverse_branches:ident, $single_owner_sublist_remove_branches:ident, $single_owner_sublist_insert_branches:ident, $single_owner_ruin_remove_branches:ident, $single_owner_ruin_insert_branches:ident, $single_owner_remove_for_construction_branches:ident, $single_owner_index_to_element_branches:ident, $single_owner_descriptor_index_branches:ident, $single_owner_element_count_branches:ident, $single_owner_assigned_elements_branches:ident, $single_owner_n_entities_branches:ident, $single_owner_assign_element_branches:ident, $total_list_entities_terms:ident, $total_list_elements_terms:ident) => {
    quote! {
        #(#$owner_helpers)*

        const __SOLVERFORGE_LIST_OWNER_COUNT: usize = 0 #(+ #$list_owner_count_terms)*;

        #[inline]
        fn __solverforge_assert_single_list_owner() {
            assert!(
                Self::__SOLVERFORGE_LIST_OWNER_COUNT == 1,
                "single-owner list helper called on a solution with {} list owners",
                Self::__SOLVERFORGE_LIST_OWNER_COUNT,
            );
        }

        #(#$owner_public_methods)*

        #[inline]
        pub fn list_len(&self, entity_idx: usize) -> usize {
            Self::list_len_static(self, entity_idx)
        }

        #[inline]
        pub fn list_len_static(s: &Self, entity_idx: usize) -> usize {
            Self::__solverforge_assert_single_list_owner();
            #(#$single_owner_list_len_branches)*
            unreachable!("single-owner list helper called without a canonical list owner");
        }

        #[inline]
        pub fn list_remove(s: &mut Self, entity_idx: usize, pos: usize) -> ::core::option::Option<usize> {
            Self::__solverforge_assert_single_list_owner();
            #(#$single_owner_list_remove_branches)*
            unreachable!("single-owner list helper called without a canonical list owner");
        }

        #[inline]
        pub fn list_insert(s: &mut Self, entity_idx: usize, pos: usize, val: usize) {
            Self::__solverforge_assert_single_list_owner();
            #(#$single_owner_list_insert_branches)*
            unreachable!("single-owner list helper called without a canonical list owner");
        }

        #[inline]
        pub fn list_get(s: &Self, entity_idx: usize, pos: usize) -> ::core::option::Option<usize> {
            Self::__solverforge_assert_single_list_owner();
            #(#$single_owner_list_get_branches)*
            unreachable!("single-owner list helper called without a canonical list owner");
        }

        #[inline]
        pub fn list_set(s: &mut Self, entity_idx: usize, pos: usize, val: usize) {
            Self::__solverforge_assert_single_list_owner();
            #(#$single_owner_list_set_branches)*
            unreachable!("single-owner list helper called without a canonical list owner");
        }

        #[inline]
        pub fn list_reverse(s: &mut Self, entity_idx: usize, start: usize, end: usize) {
            Self::__solverforge_assert_single_list_owner();
            #(#$single_owner_list_reverse_branches)*
            unreachable!("single-owner list helper called without a canonical list owner");
        }

        #[inline]
        pub fn sublist_remove(
            s: &mut Self,
            entity_idx: usize,
            start: usize,
            end: usize,
        ) -> Vec<usize> {
            Self::__solverforge_assert_single_list_owner();
            #(#$single_owner_sublist_remove_branches)*
            unreachable!("single-owner list helper called without a canonical list owner");
        }

        #[inline]
        pub fn sublist_insert(
            s: &mut Self,
            entity_idx: usize,
            pos: usize,
            items: Vec<usize>,
        ) {
            Self::__solverforge_assert_single_list_owner();
            #(#$single_owner_sublist_insert_branches)*
            unreachable!("single-owner list helper called without a canonical list owner");
        }

        #[inline]
        pub fn ruin_remove(s: &mut Self, entity_idx: usize, pos: usize) -> usize {
            Self::__solverforge_assert_single_list_owner();
            #(#$single_owner_ruin_remove_branches)*
            unreachable!("single-owner list helper called without a canonical list owner");
        }

        #[inline]
        pub fn ruin_insert(s: &mut Self, entity_idx: usize, pos: usize, val: usize) {
            Self::__solverforge_assert_single_list_owner();
            #(#$single_owner_ruin_insert_branches)*
            unreachable!("single-owner list helper called without a canonical list owner");
        }

        #[inline]
        pub fn list_remove_for_construction(s: &mut Self, entity_idx: usize, pos: usize) -> usize {
            Self::__solverforge_assert_single_list_owner();
            #(#$single_owner_remove_for_construction_branches)*
            unreachable!("single-owner list helper called without a canonical list owner");
        }

        #[inline]
        pub fn index_to_element_static(s: &Self, idx: usize) -> usize {
            Self::__solverforge_assert_single_list_owner();
            #(#$single_owner_index_to_element_branches)*
            unreachable!("single-owner list helper called without a canonical list owner");
        }

        #[inline]
        pub fn list_variable_descriptor_index() -> usize {
            Self::__solverforge_assert_single_list_owner();
            #(#$single_owner_descriptor_index_branches)*
            unreachable!("single-owner list helper called without a canonical list owner");
        }

        #[inline]
        pub fn element_count(s: &Self) -> usize {
            Self::__solverforge_assert_single_list_owner();
            #(#$single_owner_element_count_branches)*
            unreachable!("single-owner list helper called without a canonical list owner");
        }

        #[inline]
        pub fn assigned_elements(s: &Self) -> Vec<usize> {
            Self::__solverforge_assert_single_list_owner();
            #(#$single_owner_assigned_elements_branches)*
            unreachable!("single-owner list helper called without a canonical list owner");
        }

        #[inline]
        pub fn n_entities(s: &Self) -> usize {
            Self::__solverforge_assert_single_list_owner();
            #(#$single_owner_n_entities_branches)*
            unreachable!("single-owner list helper called without a canonical list owner");
        }

        #[inline]
        pub fn assign_element(s: &mut Self, entity_idx: usize, elem: usize) {
            Self::__solverforge_assert_single_list_owner();
            #(#$single_owner_assign_element_branches)*
            unreachable!("single-owner list helper called without a canonical list owner");
        }

        #[inline]
        fn __solverforge_total_list_entities(s: &Self) -> usize {
            0 #(+ #$total_list_entities_terms)*
        }

        #[inline]
        fn __solverforge_total_list_elements(s: &Self) -> usize {
            0 #(+ #$total_list_elements_terms)*
        }
    }
    };
}
