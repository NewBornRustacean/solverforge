use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Ident;

use super::type_helpers::extract_collection_inner_type;
use crate::attr_parse::has_attribute;

pub(super) fn generate_runtime_solve_internal(
    constraints_path: &Option<String>,
    config_path: &Option<String>,
    solver_toml_path: &Option<String>,
) -> TokenStream {
    let Some(path) = constraints_path.as_ref() else {
        return TokenStream::new();
    };

    let constraints_fn: syn::Path =
        syn::parse_str(path).expect("constraints path must be a valid Rust path");
    let base_config_expr = if let Some(solver_toml_path) = solver_toml_path.as_ref() {
        quote! {{
            static CONFIG: ::std::sync::OnceLock<::solverforge::SolverConfig> =
                ::std::sync::OnceLock::new();
            CONFIG
                .get_or_init(|| {
                    ::solverforge::SolverConfig::from_toml_str(include_str!(#solver_toml_path))
                        .expect("embedded solver.toml must be valid")
                })
                .clone()
        }}
    } else {
        quote! { ::solverforge::__internal::load_solver_config() }
    };
    let solve_expr = if config_path.is_some() || solver_toml_path.is_some() {
        let config_expr = if let Some(config_path) = config_path.as_ref() {
            let config_fn: syn::Path =
                syn::parse_str(config_path).expect("config path must be a valid Rust path");
            quote! {
                let base_config = #base_config_expr;
                let config = #config_fn(&self, base_config);
            }
        } else {
            quote! {
                let config = #base_config_expr;
            }
        };
        quote! {
            #config_expr
            ::solverforge::__internal::run_solver_with_config(
                self,
                #constraints_fn,
                Self::descriptor,
                Self::entity_count,
                runtime,
                config,
                Self::__solverforge_default_time_limit_secs(),
                Self::__solverforge_is_trivial,
                Self::__solverforge_log_scale,
                Self::__solverforge_build_phases,
            )
        }
    } else {
        quote! {
            ::solverforge::__internal::run_solver(
                self,
                #constraints_fn,
                Self::descriptor,
                Self::entity_count,
                runtime,
                Self::__solverforge_default_time_limit_secs(),
                Self::__solverforge_is_trivial,
                Self::__solverforge_log_scale,
                Self::__solverforge_build_phases,
            )
        }
    };
    quote! {
        fn solve_internal(
            self,
            runtime: ::solverforge::SolverRuntime<Self>,
        ) -> Self {
            ::solverforge::__internal::init_console();

            #solve_expr
        }
    }
}

fn generate_scalar_runtime_setup(
    fields: &syn::punctuated::Punctuated<syn::Field, syn::token::Comma>,
    solution_name: &Ident,
) -> TokenStream {
    let entity_fields: Vec<_> = fields
        .iter()
        .filter(|f| has_attribute(&f.attrs, "planning_entity_collection"))
        .enumerate()
        .filter_map(|(idx, field)| {
            let field_name = field.ident.as_ref()?;
            let field_type = extract_collection_inner_type(&field.ty)?;
            let syn::Type::Path(type_path) = field_type else {
                return None;
            };
            let _ = type_path.path.segments.last()?;
            Some((idx, field_name, field_type))
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
    let provider_count_arms: Vec<_> = provider_fields
        .iter()
        .enumerate()
        .map(|(idx, field_name)| {
            quote! { #idx => solution.#field_name.len(), }
        })
        .collect();

    let entity_helpers: Vec<_> = entity_fields
        .iter()
        .map(|(_, field_name, field_type)| {
            let count_fn_ident = format_ident!("__solverforge_scalar_count_{}", field_name);
            let getter_ident = format_ident!("__solverforge_scalar_get_{}", field_name);
            let setter_ident = format_ident!("__solverforge_scalar_set_{}", field_name);
            let values_ident = format_ident!("__solverforge_scalar_values_{}", field_name);
            quote! {
                fn #count_fn_ident(solution: &#solution_name) -> usize {
                    solution.#field_name.len()
                }

                fn #getter_ident(
                    solution: &#solution_name,
                    entity_index: usize,
                    variable_index: usize,
                ) -> ::core::option::Option<usize> {
                    <#field_type>::__solverforge_scalar_get_by_index(
                        &solution.#field_name[entity_index],
                        variable_index,
                    )
                }

                fn #setter_ident(
                    solution: &mut #solution_name,
                    entity_index: usize,
                    variable_index: usize,
                    value: ::core::option::Option<usize>,
                ) {
                    <#field_type>::__solverforge_scalar_set_by_index(
                        &mut solution.#field_name[entity_index],
                        variable_index,
                        value,
                    );
                }

                fn #values_ident(
                    solution: &#solution_name,
                    entity_index: usize,
                    variable_index: usize,
                ) -> &[usize] {
                    <#field_type>::__solverforge_scalar_values_by_index(
                        &solution.#field_name[entity_index],
                        variable_index,
                    )
                }
            }
        })
        .collect();

    let scalar_context_pushes: Vec<_> = entity_fields
        .iter()
        .map(|(descriptor_index, field_name, field_type)| {
            let entity_count_fn_ident = format_ident!("__solverforge_scalar_count_{}", field_name);
            let getter_ident = format_ident!("__solverforge_scalar_get_{}", field_name);
            let setter_ident = format_ident!("__solverforge_scalar_set_{}", field_name);
            let values_ident = format_ident!("__solverforge_scalar_values_{}", field_name);
            quote! {
                {
                    let __solverforge_descriptor_index = #descriptor_index;
                    let __solverforge_entity_descriptor = descriptor
                        .entity_descriptors
                        .get(__solverforge_descriptor_index)
                        .expect("entity descriptor missing for scalar runtime setup");
                    for __solverforge_variable_index in 0..<#field_type>::__solverforge_scalar_variable_count() {
                        let Some(__solverforge_variable_name) =
                            <#field_type>::__solverforge_scalar_variable_name_by_index(
                                __solverforge_variable_index,
                            )
                        else {
                            continue;
                        };
                        let Some(__solverforge_variable_descriptor) = __solverforge_entity_descriptor
                            .genuine_variable_descriptors()
                            .find(|variable| {
                                variable.name == __solverforge_variable_name
                                    && variable.usize_getter.is_some()
                                    && variable.usize_setter.is_some()
                            })
                        else {
                            continue;
                        };

                        let __solverforge_value_source = if __solverforge_variable_descriptor
                            .entity_value_provider
                            .is_some()
                            || <#field_type>::__solverforge_scalar_provider_is_entity_field_by_index(
                                __solverforge_variable_index,
                            )
                        {
                            ::solverforge::__internal::ValueSource::EntitySlice {
                                values_for_entity: #values_ident,
                            }
                        } else {
                            match &__solverforge_variable_descriptor.value_range_type {
                                ::solverforge::__internal::ValueRangeType::CountableRange { from, to } => {
                                    let from = usize::try_from(*from).expect(
                                        "countable_range start must be non-negative for canonical scalar solving",
                                    );
                                    let to = usize::try_from(*to).expect(
                                        "countable_range end must be non-negative for canonical scalar solving",
                                    );
                                    ::solverforge::__internal::ValueSource::CountableRange { from, to }
                                }
                                _ => {
                                    if let Some(provider_name) =
                                        __solverforge_variable_descriptor.value_range_provider
                                    {
                                        let provider_index = __solverforge_scalar_provider_fields
                                            .iter()
                                            .position(|field| *field == provider_name)
                                            .expect("scalar value range provider must be a solution collection");
                                        ::solverforge::__internal::ValueSource::SolutionCount {
                                            count_fn: __solverforge_scalar_collection_count,
                                            provider_index,
                                        }
                                    } else {
                                        ::solverforge::__internal::ValueSource::Empty
                                    }
                                }
                            }
                        };

                        let __solverforge_context =
                            ::solverforge::__internal::ScalarVariableContext::new(
                                __solverforge_descriptor_index,
                                __solverforge_variable_index,
                                __solverforge_entity_descriptor.type_name,
                                #entity_count_fn_ident,
                                __solverforge_variable_name,
                                #getter_ident,
                                #setter_ident,
                                __solverforge_value_source,
                                __solverforge_variable_descriptor.allows_unassigned,
                            );
                        let __solverforge_context =
                            <#solution_name as ::solverforge::__internal::PlanningModelSupport>::attach_runtime_scalar_hooks(
                                __solverforge_context,
                            );
                        __solverforge_variables.push(
                            ::solverforge::__internal::VariableContext::Scalar(
                                __solverforge_context,
                            )
                        );
                    }
                }
            }
        })
        .collect();

    quote! {
        let mut __solverforge_variables = ::std::vec::Vec::new();
        let __solverforge_scalar_provider_fields: &[&str] = &[
            #(#provider_names),*
        ];
        #(#entity_helpers)*
        fn __solverforge_scalar_collection_count(
            solution: &#solution_name,
            provider_index: usize,
        ) -> usize {
            match provider_index {
                #(#provider_count_arms)*
                _ => 0,
            }
        }
        #(#scalar_context_pushes)*
    }
}

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

pub(super) fn generate_runtime_phase_support(
    fields: &syn::punctuated::Punctuated<syn::Field, syn::token::Comma>,
    constraints_path: &Option<String>,
    solution_name: &Ident,
) -> TokenStream {
    if constraints_path.is_none() {
        return TokenStream::new();
    }

    let list_owners: Vec<_> = fields
        .iter()
        .filter(|f| has_attribute(&f.attrs, "planning_entity_collection"))
        .enumerate()
        .filter_map(|(idx, field)| {
            let field_ident = field.ident.as_ref()?;
            let field_type = extract_collection_inner_type(&field.ty)?;
            let syn::Type::Path(type_path) = &field_type else {
                return None;
            };
            let type_name = type_path.path.segments.last()?.ident.to_string();
            Some((idx, field_ident, field_type, type_name))
        })
        .collect();
    let scalar_setup = generate_scalar_runtime_setup(fields, solution_name);
    let scalar_candidate_count_helper =
        generate_scalar_candidate_count_helper(fields, solution_name);

    if !list_owners.is_empty() {
        let cross_enum_ident = format_ident!("__{}CrossDistanceMeter", solution_name);
        let intra_enum_ident = format_ident!("__{}IntraDistanceMeter", solution_name);
        let has_list_variable_terms: Vec<_> = list_owners
            .iter()
            .map(|(_, _, field_type, _)| {
                let list_trait =
                    quote! { <#field_type as ::solverforge::__internal::ListVariableEntity<#solution_name>> };
                quote! { #list_trait::HAS_LIST_VARIABLE }
            })
            .collect();

        let cross_variants: Vec<_> = list_owners
            .iter()
            .map(|(idx, _, field_type, _)| {
                let variant = format_ident!("Entity{idx}");
                quote! {
                    #variant(
                        <#field_type as ::solverforge::__internal::ListVariableEntity<#solution_name>>::CrossDistanceMeter
                    )
                }
            })
            .collect();
        let intra_variants: Vec<_> = list_owners
            .iter()
            .map(|(idx, _, field_type, _)| {
                let variant = format_ident!("Entity{idx}");
                quote! {
                    #variant(
                        <#field_type as ::solverforge::__internal::ListVariableEntity<#solution_name>>::IntraDistanceMeter
                    )
                }
            })
            .collect();
        let cross_match_arms: Vec<_> = list_owners
            .iter()
            .map(|(idx, _, _, _)| {
                let variant = format_ident!("Entity{idx}");
                quote! {
                    Self::#variant(meter) => meter.distance(solution, src_entity, src_pos, dst_entity, dst_pos),
                }
            })
            .collect();
        let intra_match_arms: Vec<_> = list_owners
            .iter()
            .map(|(idx, _, _, _)| {
                let variant = format_ident!("Entity{idx}");
                quote! {
                    Self::#variant(meter) => meter.distance(solution, src_entity, src_pos, dst_entity, dst_pos),
                }
            })
            .collect();
        let list_runtime_setup: Vec<_> = list_owners
            .iter()
            .map(|(idx, field_ident, field_type, _type_name)| {
                let field_name = field_ident.to_string();
                let variant = format_ident!("Entity{idx}");
                let descriptor_index_lit =
                    syn::LitInt::new(&idx.to_string(), proc_macro2::Span::call_site());
                let list_trait = quote! {
                    <#field_type as ::solverforge::__internal::ListVariableEntity<#solution_name>>
                };
                let list_len_ident = format_ident!("__solverforge_list_len_{}", field_name);
                let list_remove_ident = format_ident!("__solverforge_list_remove_{}", field_name);
                let list_insert_ident = format_ident!("__solverforge_list_insert_{}", field_name);
                let list_get_ident = format_ident!("__solverforge_list_get_{}", field_name);
                let list_set_ident = format_ident!("__solverforge_list_set_{}", field_name);
                let list_reverse_ident =
                    format_ident!("__solverforge_list_reverse_{}", field_name);
                let sublist_remove_ident =
                    format_ident!("__solverforge_sublist_remove_{}", field_name);
                let sublist_insert_ident =
                    format_ident!("__solverforge_sublist_insert_{}", field_name);
                let ruin_remove_ident = format_ident!("__solverforge_ruin_remove_{}", field_name);
                let ruin_insert_ident = format_ident!("__solverforge_ruin_insert_{}", field_name);
                let n_entities_ident = format_ident!("__solverforge_n_entities_{}", field_name);
                let element_count_ident =
                    format_ident!("__solverforge_element_count_{}", field_name);
                let assigned_elements_ident =
                    format_ident!("__solverforge_assigned_elements_{}", field_name);
                let list_remove_for_construction_ident = format_ident!(
                    "__solverforge_list_remove_for_construction_{}",
                    field_name
                );
                let index_to_element_ident =
                    format_ident!("__solverforge_index_to_element_{}", field_name);
                quote! {
                    if #list_trait::HAS_LIST_VARIABLE {
                        let __solverforge_entity_type_name = descriptor
                            .entity_descriptors
                            .get(#descriptor_index_lit)
                            .expect("entity descriptor missing for list runtime setup")
                            .type_name;
                        let metadata = #list_trait::list_metadata();
                        __solverforge_variables.push(
                            ::solverforge::__internal::VariableContext::List(
                                ::solverforge::__internal::ListVariableContext::new(
                                    __solverforge_entity_type_name,
                                    Self::#element_count_ident,
                                    Self::#assigned_elements_ident,
                                    Self::#list_len_ident,
                                    Self::#list_remove_ident,
                                    Self::#list_remove_for_construction_ident,
                                    Self::#list_insert_ident,
                                    Self::#list_get_ident,
                                    Self::#list_set_ident,
                                    Self::#list_reverse_ident,
                                    Self::#sublist_remove_ident,
                                    Self::#sublist_insert_ident,
                                    Self::#ruin_remove_ident,
                                    Self::#ruin_insert_ident,
                                    Self::#index_to_element_ident,
                                    Self::#n_entities_ident,
                                    #cross_enum_ident::#variant(metadata.cross_distance_meter.clone()),
                                    #intra_enum_ident::#variant(metadata.intra_distance_meter.clone()),
                                    #list_trait::LIST_VARIABLE_NAME,
                                    #descriptor_index_lit,
                                    metadata.merge_feasible_fn,
                                    metadata.cw_depot_fn,
                                    metadata.cw_distance_fn,
                                    metadata.cw_element_load_fn,
                                    metadata.cw_capacity_fn,
                                    metadata.cw_assign_route_fn,
                                    metadata.k_opt_get_route,
                                    metadata.k_opt_set_route,
                                    metadata.k_opt_depot_fn,
                                    metadata.k_opt_distance_fn,
                                    metadata.k_opt_feasible_fn,
                                )
                            )
                        );
                    }
                }
            })
            .collect();

        return quote! {
            #[derive(Clone, Debug)]
            enum #cross_enum_ident {
                #(#cross_variants),*
            }

            impl ::solverforge::CrossEntityDistanceMeter<#solution_name> for #cross_enum_ident {
                fn distance(
                    &self,
                    solution: &#solution_name,
                    src_entity: usize,
                    src_pos: usize,
                    dst_entity: usize,
                    dst_pos: usize,
                ) -> f64 {
                    match self {
                        #(#cross_match_arms)*
                    }
                }
            }

            #[derive(Clone, Debug)]
            enum #intra_enum_ident {
                #(#intra_variants),*
            }

            impl ::solverforge::CrossEntityDistanceMeter<#solution_name> for #intra_enum_ident {
                fn distance(
                    &self,
                    solution: &#solution_name,
                    src_entity: usize,
                    src_pos: usize,
                    dst_entity: usize,
                    dst_pos: usize,
                ) -> f64 {
                    match self {
                        #(#intra_match_arms)*
                    }
                }
            }

            impl #solution_name {
                #scalar_candidate_count_helper

                fn __solverforge_default_time_limit_secs() -> u64 {
                    if Self::__solverforge_has_list_variable() {
                        60
                    } else {
                        30
                    }
                }

                #[inline]
                fn __solverforge_has_list_variable() -> bool {
                    false #(|| #has_list_variable_terms)*
                }

                fn __solverforge_is_trivial(solution: &Self) -> bool {
                    let descriptor = Self::descriptor();
                    let has_scalar = ::solverforge::__internal::descriptor_has_bindings(&descriptor);
                    let total_entity_count = descriptor
                        .total_entity_count(solution as &dyn ::std::any::Any)
                        .unwrap_or(0);
                    if total_entity_count == 0 {
                        return true;
                    }

                    if !Self::__solverforge_has_list_variable() {
                        return !has_scalar;
                    }

                    let has_list = Self::__solverforge_total_list_entities(solution) > 0
                        && Self::__solverforge_total_list_elements(solution) > 0;
                    !has_scalar && !has_list
                }

                fn __solverforge_log_scale(solution: &Self) {
                    let descriptor = Self::descriptor();
                    if Self::__solverforge_has_list_variable() {
                        ::solverforge::__internal::log_solve_start(
                            Self::__solverforge_total_list_entities(solution),
                            ::core::option::Option::Some(
                                Self::__solverforge_total_list_elements(solution),
                            ),
                            ::core::option::Option::None,
                        );
                    } else {
                        ::solverforge::__internal::log_solve_start(
                            descriptor
                                .total_entity_count(solution as &dyn ::std::any::Any)
                                .unwrap_or(0),
                            ::core::option::Option::None,
                            ::core::option::Option::Some(
                                Self::__solverforge_scalar_candidate_count(solution),
                            ),
                        );
                    }
                }

                fn __solverforge_build_phases(
                    config: &::solverforge::__internal::SolverConfig,
                ) -> ::solverforge::__internal::PhaseSequence<
                    ::solverforge::__internal::RuntimePhase<
                        ::solverforge::__internal::Construction<
                            #solution_name,
                            usize,
                            #cross_enum_ident,
                            #intra_enum_ident
                        >,
                        ::solverforge::__internal::LocalSearch<
                            #solution_name,
                            usize,
                            #cross_enum_ident,
                            #intra_enum_ident
                        >,
                        ::solverforge::__internal::Vnd<
                            #solution_name,
                            usize,
                            #cross_enum_ident,
                            #intra_enum_ident
                        >
                    >
                > {
                    let descriptor = Self::descriptor();
                    #scalar_setup
                    #(#list_runtime_setup)*
                    let __solverforge_descriptor_variable_order =
                        |descriptor_index: usize, variable_name: &str| {
                            descriptor
                                .entity_descriptors
                                .get(descriptor_index)
                                .and_then(|entity| {
                                    entity
                                        .variable_descriptors
                                        .iter()
                                        .position(|descriptor_var| {
                                            descriptor_var.name == variable_name
                                        })
                                })
                                .unwrap_or(::core::usize::MAX)
                        };
                    __solverforge_variables.sort_by_key(|variable| {
                        match variable {
                            ::solverforge::__internal::VariableContext::Scalar(ctx) => {
                                (
                                    ctx.descriptor_index,
                                    __solverforge_descriptor_variable_order(
                                        ctx.descriptor_index,
                                        ctx.variable_name,
                                    ),
                                )
                            }
                            ::solverforge::__internal::VariableContext::List(ctx) => {
                                (
                                    ctx.descriptor_index,
                                    __solverforge_descriptor_variable_order(
                                        ctx.descriptor_index,
                                        ctx.variable_name,
                                    ),
                                )
                            }
                        }
                    });
                    let model = ::solverforge::__internal::ModelContext::<
                        #solution_name,
                        usize,
                        #cross_enum_ident,
                        #intra_enum_ident
                    >::new(__solverforge_variables);
                    ::solverforge::__internal::build_phases(
                        config,
                        &descriptor,
                        &model,
                    )
                }
            }
        };
    }

    quote! {
        impl #solution_name {
            #scalar_candidate_count_helper

            const fn __solverforge_default_time_limit_secs() -> u64 {
                30
            }

            fn __solverforge_is_trivial(solution: &Self) -> bool {
                let descriptor = Self::descriptor();
                !::solverforge::__internal::descriptor_has_bindings(&descriptor)
                    || descriptor
                        .total_entity_count(solution as &dyn ::std::any::Any)
                        .unwrap_or(0)
                        == 0
            }

            fn __solverforge_log_scale(solution: &Self) {
                let descriptor = Self::descriptor();
                ::solverforge::__internal::log_solve_start(
                    descriptor
                        .total_entity_count(solution as &dyn ::std::any::Any)
                        .unwrap_or(0),
                    ::core::option::Option::None,
                    ::core::option::Option::Some(
                        Self::__solverforge_scalar_candidate_count(solution),
                    ),
                );
            }

            fn __solverforge_build_phases(
                config: &::solverforge::__internal::SolverConfig,
            ) -> ::solverforge::__internal::PhaseSequence<
                ::solverforge::__internal::RuntimePhase<
                    ::solverforge::__internal::Construction<
                        #solution_name,
                        usize,
                        ::solverforge::__internal::DefaultCrossEntityDistanceMeter,
                        ::solverforge::__internal::DefaultCrossEntityDistanceMeter
                    >,
                    ::solverforge::__internal::LocalSearch<
                        #solution_name,
                        usize,
                        ::solverforge::__internal::DefaultCrossEntityDistanceMeter,
                        ::solverforge::__internal::DefaultCrossEntityDistanceMeter
                    >,
                    ::solverforge::__internal::Vnd<
                        #solution_name,
                        usize,
                        ::solverforge::__internal::DefaultCrossEntityDistanceMeter,
                        ::solverforge::__internal::DefaultCrossEntityDistanceMeter
                    >
                >
            > {
                let descriptor = Self::descriptor();
                #scalar_setup
                let model = ::solverforge::__internal::ModelContext::<
                    #solution_name,
                    usize,
                    ::solverforge::__internal::DefaultCrossEntityDistanceMeter,
                    ::solverforge::__internal::DefaultCrossEntityDistanceMeter
                >::new(__solverforge_variables);
                ::solverforge::__internal::build_phases(
                    config,
                    &descriptor,
                    &model,
                )
            }
        }
    }
}

pub(super) fn generate_solvable_solution(
    solution_name: &Ident,
    constraints_path: &Option<String>,
) -> TokenStream {
    let solvable_solution_impl = quote! {
        impl ::solverforge::__internal::SolvableSolution for #solution_name {
            fn descriptor() -> ::solverforge::__internal::SolutionDescriptor {
                #solution_name::descriptor()
            }

            fn entity_count(solution: &Self, descriptor_index: usize) -> usize {
                #solution_name::entity_count(solution, descriptor_index)
            }
        }
    };

    let solvable_impl = constraints_path.as_ref().map(|path| {
        let constraints_fn: syn::Path =
            syn::parse_str(path).expect("constraints path must be a valid Rust path");

        quote! {
            impl ::solverforge::Solvable for #solution_name {
                fn solve(
                    self,
                    runtime: ::solverforge::SolverRuntime<Self>,
                ) {
                    let _ = #solution_name::solve_internal(self, runtime);
                }
            }

            impl ::solverforge::Analyzable for #solution_name {
                fn analyze(&self) -> ::solverforge::ScoreAnalysis<<Self as ::solverforge::__internal::PlanningSolution>::Score> {
                    use ::solverforge::__internal::{
                        Director, ScoreDirector,
                    };

                    let constraints = #constraints_fn();
                    let mut director = ScoreDirector::with_descriptor(
                        self.clone(),
                        constraints,
                        Self::descriptor(),
                        Self::entity_count,
                    );

                    let score = director.calculate_score();
                    let constraint_scores = director.constraint_match_totals();

                    let constraints = constraint_scores
                        .into_iter()
                        .map(|(name, weight, contribution, match_count)| {
                            ::solverforge::ConstraintAnalysis {
                                name,
                                weight,
                                score: contribution,
                                match_count,
                            }
                        })
                        .collect();

                    ::solverforge::ScoreAnalysis { score, constraints }
                }
            }
        }
    });

    quote! {
        #solvable_solution_impl
        #solvable_impl
    }
}
