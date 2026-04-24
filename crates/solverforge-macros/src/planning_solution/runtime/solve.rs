
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

