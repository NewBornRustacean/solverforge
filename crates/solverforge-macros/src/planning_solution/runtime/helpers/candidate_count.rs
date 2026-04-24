fn generate_scalar_candidate_count_helper(
    fields: &syn::punctuated::Punctuated<syn::Field, syn::token::Comma>,
    solution_name: &Ident,
) -> TokenStream {
    let entity_fields: Vec<_> = fields
        .iter()
        .filter(|f| has_attribute(&f.attrs, "planning_entity_collection"))
        .enumerate()
        .filter_map(|field| {
            let (descriptor_index, field) = field;
            let field_name = field.ident.as_ref()?;
            let field_type = extract_collection_inner_type(&field.ty)?;
            let syn::Type::Path(_type_path) = field_type else {
                return None;
            };
            Some((descriptor_index, field_name, field_type))
        })
        .collect();

    let provider_fields: Vec<_> = fields
        .iter()
        .filter(|f| {
            has_attribute(&f.attrs, "planning_entity_collection")
                || has_attribute(&f.attrs, "problem_fact_collection")
        })
        .filter_map(|field| field.ident.as_ref())
        .collect();

    let provider_names: Vec<_> = provider_fields
        .iter()
        .map(|field_name| field_name.to_string())
        .collect();

    let candidate_accumulators: Vec<_> = entity_fields
        .iter()
        .map(|(descriptor_index, field_name, field_type)| {
            quote! {
                if let Some(__solverforge_entity_descriptor) =
                    descriptor.entity_descriptors.get(#descriptor_index)
                {
                    for __solverforge_variable_index in 0..<#field_type>::__solverforge_scalar_variable_count() {
                        let Some(__solverforge_variable_name) =
                            <#field_type>::__solverforge_scalar_variable_name_by_index(
                                __solverforge_variable_index,
                            )
                        else {
                            continue;
                        };
                        let Some(__solverforge_variable_descriptor) =
                            __solverforge_entity_descriptor
                                .genuine_variable_descriptors()
                                .find(|variable| {
                                    variable.name == __solverforge_variable_name
                                        && variable.usize_getter.is_some()
                                        && variable.usize_setter.is_some()
                                })
                        else {
                            continue;
                        };

                        let slot_count = solution.#field_name.len();
                        total_slots += slot_count;
                        if __solverforge_variable_descriptor.entity_value_provider.is_some()
                            || <#field_type>::__solverforge_scalar_provider_is_entity_field_by_index(
                                __solverforge_variable_index,
                            )
                        {
                            total_candidates += solution
                                .#field_name
                                .iter()
                                .map(|entity| {
                                    <#field_type>::__solverforge_scalar_values_by_index(
                                        entity,
                                        __solverforge_variable_index,
                                    )
                                    .len()
                                })
                                .sum::<usize>();
                        } else {
                            match &__solverforge_variable_descriptor.value_range_type {
                                ::solverforge::__internal::ValueRangeType::CountableRange { from, to } => {
                                    let from = usize::try_from(*from).expect(
                                        "countable_range start must be non-negative for candidate counting",
                                    );
                                    let to = usize::try_from(*to).expect(
                                        "countable_range end must be non-negative for candidate counting",
                                    );
                                    total_candidates += slot_count * to.saturating_sub(from);
                                }
                                _ => {
                                    if let Some(provider_name) =
                                        __solverforge_variable_descriptor.value_range_provider
                                    {
                                        if let Some(provider_index) = __solverforge_scalar_provider_fields
                                            .iter()
                                            .position(|field| *field == provider_name)
                                        {
                                            total_candidates += slot_count
                                                * Self::__solverforge_scalar_collection_count(
                                                    solution,
                                                    provider_index,
                                                );
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        })
        .collect();

    let provider_count_arms: Vec<_> = provider_fields
        .iter()
        .enumerate()
        .map(|(idx, field_name)| {
            quote! { #idx => solution.#field_name.len(), }
        })
        .collect();

    quote! {
        fn __solverforge_scalar_collection_count(
            solution: &#solution_name,
            provider_index: usize,
        ) -> usize {
            match provider_index {
                #(#provider_count_arms)*
                _ => 0,
            }
        }

        fn __solverforge_scalar_candidate_count(solution: &#solution_name) -> usize {
            let descriptor = Self::descriptor();
            let __solverforge_scalar_provider_fields: &[&str] = &[
                #(#provider_names),*
            ];
            let mut total_slots = 0usize;
            let mut total_candidates = 0usize;
            #(#candidate_accumulators)*
            if total_slots == 0 {
                0
            } else {
                (total_candidates + (total_slots / 2)) / total_slots
            }
        }
    }
}

