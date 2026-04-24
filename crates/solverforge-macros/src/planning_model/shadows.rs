fn generate_shadow_methods(model: &ModelMetadata) -> Result<TokenStream> {
    let solution_module = &model.solution.module_ident;
    let solution_ident = &model.solution.ident;
    let solution_path = quote! { #solution_module::#solution_ident };
    let config = &model.solution.shadow_config;
    if !shadow_updates_requested(config) {
        return Ok(quote! {
            fn update_entity_shadows(
                _solution: &mut Self,
                _descriptor_index: usize,
                _entity_index: usize,
            ) -> bool {
                false
            }

            fn update_all_shadows(_solution: &mut Self) -> bool {
                false
            }
        });
    }

    let list_owner = config.list_owner.as_deref().ok_or_else(|| {
        Error::new(
            proc_macro2::Span::call_site(),
            "#[shadow_variable_updates(...)] requires `list_owner = \"entity_collection_field\"` when shadow updates are configured",
        )
    })?;
    let list_owner_collection = model
        .solution
        .collections
        .iter()
        .find(|collection| {
            collection.field_name == list_owner && collection.descriptor_index.is_some()
        })
        .ok_or_else(|| {
            Error::new(
                proc_macro2::Span::call_site(),
                format!(
                    "#[shadow_variable_updates(list_owner = \"{list_owner}\")] must name a #[planning_entity_collection] field",
                ),
            )
        })?;
    let list_owner_ident = &list_owner_collection.field_ident;
    let list_owner_accessor = format_ident!("__solverforge_collection_{}", list_owner_ident);
    let list_owner_mut_accessor =
        format_ident!("__solverforge_collection_{}_mut", list_owner_ident);
    let list_owner_descriptor_index = list_owner_collection.descriptor_index.unwrap();
    let entity_type_name = canonical_type_name(&model.aliases, &list_owner_collection.type_name);
    let entity = model
        .entities
        .get(entity_type_name)
        .expect("list owner entity should have been validated");
    let element_collection_name = entity.list_element_collection.as_deref().ok_or_else(|| {
        Error::new(
            proc_macro2::Span::call_site(),
            format!("list owner `{list_owner}` does not declare #[planning_list_variable]"),
        )
    })?;
    let list_variable_ident = entity
        .list_variable_name
        .as_deref()
        .map(|name| Ident::new(name, proc_macro2::Span::call_site()))
        .ok_or_else(|| {
            Error::new(
                proc_macro2::Span::call_site(),
                format!("list owner `{list_owner}` does not declare #[planning_list_variable]"),
            )
        })?;
    let element_collection = model
        .solution
        .collections
        .iter()
        .find(|collection| collection.field_name == element_collection_name)
        .ok_or_else(|| {
            Error::new(
                proc_macro2::Span::call_site(),
                format!(
                    "planning solution with list owner `{list_owner}` requires a `#[planning_entity_collection]` or `#[problem_fact_collection]` field named `{element_collection_name}`",
                ),
            )
        })?;
    let element_collection_ident = &element_collection.field_ident;
    let element_collection_accessor =
        format_ident!("__solverforge_collection_{}", element_collection_ident);
    let element_collection_mut_accessor =
        format_ident!("__solverforge_collection_{}_mut", element_collection_ident);

    let inverse_update = config.inverse_field.as_ref().map(|field| {
        let field_ident = Ident::new(field, proc_macro2::Span::call_site());
        quote! {
            {
                let elements = #solution_path::#element_collection_mut_accessor(solution);
                for &element_idx in &element_indices {
                    elements[element_idx].#field_ident = Some(entity_index);
                }
            }
        }
    });

    let previous_update = config.previous_field.as_ref().map(|field| {
        let field_ident = Ident::new(field, proc_macro2::Span::call_site());
        quote! {
            {
                let elements = #solution_path::#element_collection_mut_accessor(solution);
                let mut prev_idx: Option<usize> = None;
                for &element_idx in &element_indices {
                    elements[element_idx].#field_ident = prev_idx;
                    prev_idx = Some(element_idx);
                }
            }
        }
    });

    let next_update = config.next_field.as_ref().map(|field| {
        let field_ident = Ident::new(field, proc_macro2::Span::call_site());
        quote! {
            {
                let elements = #solution_path::#element_collection_mut_accessor(solution);
                let len = element_indices.len();
                for (i, &element_idx) in element_indices.iter().enumerate() {
                    let next_idx = if i + 1 < len { Some(element_indices[i + 1]) } else { None };
                    elements[element_idx].#field_ident = next_idx;
                }
            }
        }
    });

    let cascading_update = config.cascading_listener.as_ref().map(|method| {
        let method_ident = Ident::new(method, proc_macro2::Span::call_site());
        quote! {
            for &element_idx in &element_indices {
                solution.#method_ident(element_idx);
            }
        }
    });

    let post_update = config.post_update_listener.as_ref().map(|method| {
        let method_ident = Ident::new(method, proc_macro2::Span::call_site());
        quote! {
            solution.#method_ident(entity_index);
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
                    let aggregate_value = {
                        let elements = #solution_path::#element_collection_accessor(solution);
                        element_indices
                            .iter()
                            .map(|&idx| elements[idx].#source_field)
                            .sum()
                    };
                    #solution_path::#list_owner_mut_accessor(solution)[entity_index].#target_field =
                        aggregate_value;
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
                let computed_value = solution.#method_name(entity_index);
                #solution_path::#list_owner_mut_accessor(solution)[entity_index].#target_field =
                    computed_value;
            })
        })
        .collect();

    Ok(quote! {
        fn update_entity_shadows(
            solution: &mut Self,
            descriptor_index: usize,
            entity_index: usize,
        ) -> bool {
            if descriptor_index != #list_owner_descriptor_index {
                return false;
            }

            let element_indices: Vec<usize> =
                #solution_path::#list_owner_accessor(solution)[entity_index].#list_variable_ident
                    .to_vec();

            #inverse_update
            #previous_update
            #next_update
            #cascading_update
            #(#aggregate_updates)*
            #(#compute_updates)*
            #post_update

            true
        }

        fn update_all_shadows(solution: &mut Self) -> bool {
            for entity_index in 0..#solution_path::#list_owner_accessor(solution).len() {
                <Self as ::solverforge::__internal::PlanningModelSupport>::update_entity_shadows(
                    solution,
                    #list_owner_descriptor_index,
                    entity_index,
                );
            }
            true
        }
    })
}

fn shadow_updates_requested(config: &ShadowConfig) -> bool {
    config.inverse_field.is_some()
        || config.previous_field.is_some()
        || config.next_field.is_some()
        || config.cascading_listener.is_some()
        || config.post_update_listener.is_some()
        || !config.entity_aggregates.is_empty()
        || !config.entity_computes.is_empty()
}

