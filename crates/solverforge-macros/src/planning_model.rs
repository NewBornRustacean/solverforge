use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::{
    Attribute, Error, Fields, Ident, Item, ItemMod, ItemStruct, ItemType, ItemUse, LitStr, Result,
    Token, Type, UseTree, Visibility,
};

use crate::attr_parse::{
    get_attribute, has_attribute, parse_attribute_bool, parse_attribute_list,
    parse_attribute_string,
};

struct PlanningModelInput {
    root: LitStr,
    items: Vec<ManifestItem>,
}

enum ManifestItem {
    Mod(ItemMod),
    Use(ItemUse),
}

impl Parse for PlanningModelInput {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let root_ident: Ident = input.parse()?;
        if root_ident != "root" {
            return Err(Error::new_spanned(root_ident, "expected `root`"));
        }
        input.parse::<Token![=]>()?;
        let root = input.parse::<LitStr>()?;
        input.parse::<Token![;]>()?;

        let mut items = Vec::new();
        while !input.is_empty() {
            let item = input.parse::<Item>()?;
            match item {
                Item::Mod(item_mod) => {
                    if item_mod.content.is_some() {
                        return Err(Error::new_spanned(
                            item_mod,
                            "planning_model! only accepts file-backed `mod name;` declarations",
                        ));
                    }
                    items.push(ManifestItem::Mod(item_mod));
                }
                Item::Use(item_use) => {
                    if !matches!(item_use.vis, Visibility::Public(_)) {
                        return Err(Error::new_spanned(
                            item_use,
                            "planning_model! only accepts public use exports",
                        ));
                    }
                    items.push(ManifestItem::Use(item_use));
                }
                other => {
                    return Err(Error::new_spanned(
                        other,
                        "planning_model! accepts only `mod name;` and `pub use ...;` items",
                    ));
                }
            }
        }

        Ok(Self { root, items })
    }
}

impl ToTokens for ManifestItem {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Mod(item) => item.to_tokens(tokens),
            Self::Use(item) => item.to_tokens(tokens),
        }
    }
}

struct ModuleSource {
    ident: Ident,
    path: PathBuf,
    file: syn::File,
}

#[derive(Clone)]
struct HookPaths {
    nearby_value_distance_meter: Option<syn::Path>,
    nearby_entity_distance_meter: Option<syn::Path>,
    construction_entity_order_key: Option<syn::Path>,
    construction_value_order_key: Option<syn::Path>,
}

#[derive(Clone)]
struct ScalarVariableMetadata {
    field_name: String,
    hooks: HookPaths,
}

#[derive(Clone)]
struct EntityMetadata {
    type_name: String,
    scalar_variables: Vec<ScalarVariableMetadata>,
    list_variable_name: Option<String>,
    list_element_collection: Option<String>,
}

struct SolutionCollection {
    field_ident: Ident,
    field_name: String,
    type_name: String,
    descriptor_index: Option<usize>,
}

struct SolutionMetadata {
    module_ident: Ident,
    ident: Ident,
    collections: Vec<SolutionCollection>,
    collection_field_names: BTreeSet<String>,
    shadow_config: ShadowConfig,
}

struct ModelMetadata {
    solution: SolutionMetadata,
    entities: BTreeMap<String, EntityMetadata>,
    aliases: BTreeMap<String, String>,
}

#[derive(Clone, Default)]
struct ShadowConfig {
    list_owner: Option<String>,
    inverse_field: Option<String>,
    previous_field: Option<String>,
    next_field: Option<String>,
    cascading_listener: Option<String>,
    post_update_listener: Option<String>,
    entity_aggregates: Vec<String>,
    entity_computes: Vec<String>,
}

pub(crate) fn expand(input: TokenStream) -> Result<TokenStream> {
    let manifest: PlanningModelInput = syn::parse2(input)?;
    let root = resolve_root(&manifest.root)?;
    let modules = read_modules(&manifest.items, &root)?;
    let model = collect_model_metadata(&manifest.items, &modules)?;
    let support_impl = generate_support_impl(&model)?;
    let module_dependency_paths = modules.iter().map(|module| {
        LitStr::new(
            &module.path.to_string_lossy(),
            proc_macro2::Span::call_site(),
        )
    });

    let items = manifest.items.iter();
    Ok(quote! {
        #(#items)*
        const _: &[&str] = &[
            #(include_str!(#module_dependency_paths)),*
        ];
        #support_impl
    })
}

fn resolve_root(root: &LitStr) -> Result<PathBuf> {
    let value = root.value();
    let root_path = PathBuf::from(&value);
    if root_path.is_absolute() {
        return Ok(root_path);
    }

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").map_err(|_| {
        Error::new_spanned(root, "planning_model! could not resolve CARGO_MANIFEST_DIR")
    })?;
    let manifest_root = PathBuf::from(&manifest_dir).join(&root_path);
    if manifest_root.exists() {
        return Ok(manifest_root);
    }
    for ancestor in PathBuf::from(&manifest_dir).ancestors() {
        let candidate = ancestor.join(&root_path);
        if candidate.exists() {
            return Ok(candidate);
        }
    }

    if let Ok(current_dir) = std::env::current_dir() {
        let current_root = current_dir.join(&root_path);
        if current_root.exists() {
            return Ok(current_root);
        }
        for ancestor in current_dir.ancestors() {
            let candidate = ancestor.join(&root_path);
            if candidate.exists() {
                return Ok(candidate);
            }
        }
    }

    Ok(PathBuf::from(manifest_dir).join(root_path))
}

fn read_modules(items: &[ManifestItem], root: &Path) -> Result<Vec<ModuleSource>> {
    let mut modules = Vec::new();
    for item in items {
        let ManifestItem::Mod(item_mod) = item else {
            continue;
        };
        let ident = item_mod.ident.clone();
        let path = module_path(root, &ident).ok_or_else(|| {
            Error::new_spanned(
                item_mod,
                format!(
                    "planning_model! module `{ident}` must resolve to `{}/{ident}.rs` or `{}/{ident}/mod.rs`",
                    root.display(),
                    root.display(),
                ),
            )
        })?;
        let source = std::fs::read_to_string(&path).map_err(|err| {
            Error::new_spanned(
                item_mod,
                format!(
                    "planning_model! could not read module `{ident}` at `{}`: {err}",
                    path.display(),
                ),
            )
        })?;
        let file = syn::parse_file(&source).map_err(|err| {
            Error::new_spanned(
                item_mod,
                format!(
                    "planning_model! could not parse module `{ident}` at `{}`: {err}",
                    path.display(),
                ),
            )
        })?;
        modules.push(ModuleSource { ident, path, file });
    }
    Ok(modules)
}

fn module_path(root: &Path, ident: &Ident) -> Option<PathBuf> {
    let file_path = root.join(format!("{ident}.rs"));
    if file_path.exists() {
        return Some(file_path);
    }
    let mod_path = root.join(ident.to_string()).join("mod.rs");
    mod_path.exists().then_some(mod_path)
}

fn collect_model_metadata(
    items: &[ManifestItem],
    modules: &[ModuleSource],
) -> Result<ModelMetadata> {
    let mut solution: Option<SolutionMetadata> = None;
    let mut entities = BTreeMap::new();
    let mut facts = BTreeSet::new();
    let mut raw_aliases = BTreeMap::new();

    for item in items {
        if let ManifestItem::Use(item_use) = item {
            collect_use_aliases(item_use, &mut raw_aliases)?;
        }
    }

    for module in modules {
        for item in &module.file.items {
            match item {
                Item::Struct(item_struct) => {
                    if has_attribute(&item_struct.attrs, "planning_solution") {
                        if let Some(existing) = &solution {
                            return Err(Error::new_spanned(
                                item_struct,
                                format!(
                                    "planning_model! found duplicate #[planning_solution]; `{}` is already the model solution",
                                    existing.ident
                                ),
                            ));
                        }
                        solution = Some(parse_solution(module, item_struct)?);
                    }
                    if has_attribute(&item_struct.attrs, "planning_entity") {
                        let metadata = parse_entity(module, item_struct)?;
                        entities.insert(metadata.type_name.clone(), metadata);
                    }
                    if has_attribute(&item_struct.attrs, "problem_fact") {
                        facts.insert(item_struct.ident.to_string());
                    }
                }
                Item::Type(item_type) => {
                    if let Some(target) = alias_target_name(item_type) {
                        insert_alias(
                            &mut raw_aliases,
                            item_type.ident.to_string(),
                            target,
                            item_type,
                        )?;
                    }
                }
                Item::Use(item_use) if matches!(item_use.vis, Visibility::Public(_)) => {
                    collect_use_aliases(item_use, &mut raw_aliases)?;
                }
                _ => {}
            }
        }
    }

    let Some(solution) = solution else {
        return Err(Error::new(
            proc_macro2::Span::call_site(),
            "planning_model! requires exactly one #[planning_solution] in the listed modules",
        ));
    };

    let aliases = resolve_aliases(&raw_aliases)?;
    validate_collections(&solution, &entities, &facts, &aliases)?;
    validate_list_element_sources(&solution, &entities, &aliases)?;

    Ok(ModelMetadata {
        solution,
        entities,
        aliases,
    })
}

fn alias_target_name(item_type: &ItemType) -> Option<String> {
    type_name(&item_type.ty)
}

fn collect_use_aliases(item_use: &ItemUse, aliases: &mut BTreeMap<String, String>) -> Result<()> {
    fn walk(
        tree: &UseTree,
        aliases: &mut BTreeMap<String, String>,
        span: &impl ToTokens,
    ) -> Result<()> {
        match tree {
            UseTree::Path(path) => walk(&path.tree, aliases, span),
            UseTree::Rename(rename) => insert_alias(
                aliases,
                rename.rename.to_string(),
                rename.ident.to_string(),
                span,
            ),
            UseTree::Group(group) => {
                for item in &group.items {
                    walk(item, aliases, span)?;
                }
                Ok(())
            }
            UseTree::Name(_) | UseTree::Glob(_) => Ok(()),
        }
    }

    walk(&item_use.tree, aliases, item_use)
}

fn insert_alias(
    aliases: &mut BTreeMap<String, String>,
    alias: String,
    target: String,
    span: &impl ToTokens,
) -> Result<()> {
    if alias == target {
        return Ok(());
    }
    if let Some(existing) = aliases.get(&alias) {
        if existing != &target {
            return Err(Error::new_spanned(
                span,
                format!(
                    "planning_model! alias `{alias}` points to both `{existing}` and `{target}`",
                ),
            ));
        }
        return Ok(());
    }
    aliases.insert(alias, target);
    Ok(())
}

fn resolve_aliases(raw_aliases: &BTreeMap<String, String>) -> Result<BTreeMap<String, String>> {
    fn resolve_one(
        name: &str,
        raw_aliases: &BTreeMap<String, String>,
        stack: &mut Vec<String>,
    ) -> Result<String> {
        let Some(target) = raw_aliases.get(name) else {
            return Ok(name.to_string());
        };
        if stack.iter().any(|seen| seen == name) {
            stack.push(name.to_string());
            return Err(Error::new(
                proc_macro2::Span::call_site(),
                format!(
                    "planning_model! alias cycle detected: {}",
                    stack.join(" -> "),
                ),
            ));
        }
        stack.push(name.to_string());
        let resolved = resolve_one(target, raw_aliases, stack)?;
        stack.pop();
        Ok(resolved)
    }

    let mut resolved = BTreeMap::new();
    for alias in raw_aliases.keys() {
        resolved.insert(
            alias.clone(),
            resolve_one(alias, raw_aliases, &mut Vec::new())?,
        );
    }
    Ok(resolved)
}

fn canonical_type_name<'a>(aliases: &'a BTreeMap<String, String>, type_name: &'a str) -> &'a str {
    aliases
        .get(type_name)
        .map(String::as_str)
        .unwrap_or(type_name)
}

fn parse_solution(module: &ModuleSource, item_struct: &ItemStruct) -> Result<SolutionMetadata> {
    let fields = named_fields(item_struct, "#[planning_solution] requires named fields")?;
    let mut collections = Vec::new();
    let mut collection_field_names = BTreeSet::new();
    let mut descriptor_index = 0usize;

    for field in fields {
        let Some(field_ident) = field.ident.clone() else {
            continue;
        };
        let field_name = field_ident.to_string();
        if has_attribute(&field.attrs, "planning_entity_collection")
            || has_attribute(&field.attrs, "problem_fact_collection")
            || has_attribute(&field.attrs, "planning_list_element_collection")
        {
            collection_field_names.insert(field_name.clone());
        }
        if has_attribute(&field.attrs, "planning_entity_collection") {
            let type_name = collection_type_name(&field.ty).ok_or_else(|| {
                Error::new_spanned(
                    field,
                    "#[planning_entity_collection] requires a Vec<T> field",
                )
            })?;
            collections.push(SolutionCollection {
                field_ident,
                field_name,
                type_name,
                descriptor_index: Some(descriptor_index),
            });
            descriptor_index += 1;
        } else if has_attribute(&field.attrs, "problem_fact_collection") {
            let type_name = collection_type_name(&field.ty).ok_or_else(|| {
                Error::new_spanned(field, "#[problem_fact_collection] requires a Vec<T> field")
            })?;
            collections.push(SolutionCollection {
                field_ident,
                field_name,
                type_name,
                descriptor_index: None,
            });
        }
    }

    Ok(SolutionMetadata {
        module_ident: module.ident.clone(),
        ident: item_struct.ident.clone(),
        collections,
        collection_field_names,
        shadow_config: parse_shadow_config(&item_struct.attrs),
    })
}

fn parse_shadow_config(attrs: &[Attribute]) -> ShadowConfig {
    let mut config = ShadowConfig::default();
    if let Some(attr) = get_attribute(attrs, "shadow_variable_updates") {
        config.list_owner = parse_attribute_string(attr, "list_owner");
        config.inverse_field = parse_attribute_string(attr, "inverse_field");
        config.previous_field = parse_attribute_string(attr, "previous_field");
        config.next_field = parse_attribute_string(attr, "next_field");
        config.cascading_listener = parse_attribute_string(attr, "cascading_listener");
        config.post_update_listener = parse_attribute_string(attr, "post_update_listener");
        config.entity_aggregates = parse_attribute_list(attr, "entity_aggregate");
        config.entity_computes = parse_attribute_list(attr, "entity_compute");
    }
    config
}

fn parse_entity(module: &ModuleSource, item_struct: &ItemStruct) -> Result<EntityMetadata> {
    let fields = named_fields(item_struct, "#[planning_entity] requires named fields")?;
    let mut scalar_variables = Vec::new();
    let mut list_variable_name = None;
    let mut list_element_collection = None;

    for field in fields {
        if has_attribute(&field.attrs, "planning_variable") {
            let Some(field_ident) = field.ident.as_ref() else {
                continue;
            };
            if !field_is_option_usize(&field.ty) {
                continue;
            }
            let attr = get_attribute(&field.attrs, "planning_variable").unwrap();
            if parse_attribute_bool(attr, "chained").unwrap_or(false) {
                continue;
            }
            scalar_variables.push(ScalarVariableMetadata {
                field_name: field_ident.to_string(),
                hooks: HookPaths {
                    nearby_value_distance_meter: parse_hook_path(
                        attr,
                        "nearby_value_distance_meter",
                        &module.ident,
                        field,
                    )?,
                    nearby_entity_distance_meter: parse_hook_path(
                        attr,
                        "nearby_entity_distance_meter",
                        &module.ident,
                        field,
                    )?,
                    construction_entity_order_key: parse_hook_path(
                        attr,
                        "construction_entity_order_key",
                        &module.ident,
                        field,
                    )?,
                    construction_value_order_key: parse_hook_path(
                        attr,
                        "construction_value_order_key",
                        &module.ident,
                        field,
                    )?,
                },
            });
        }

        if has_attribute(&field.attrs, "planning_list_variable") {
            if let Some(field_ident) = field.ident.as_ref() {
                list_variable_name = Some(field_ident.to_string());
            }
            let attr = get_attribute(&field.attrs, "planning_list_variable").unwrap();
            let element_collection =
                parse_attribute_string(attr, "element_collection").ok_or_else(|| {
                    Error::new_spanned(
                        field,
                        "#[planning_list_variable] requires `element_collection = \"solution_field\"`",
                    )
                })?;
            list_element_collection = Some(element_collection);
        }
    }

    Ok(EntityMetadata {
        type_name: item_struct.ident.to_string(),
        scalar_variables,
        list_variable_name,
        list_element_collection,
    })
}

fn named_fields<'a>(
    item_struct: &'a ItemStruct,
    message: &'static str,
) -> Result<&'a syn::punctuated::Punctuated<syn::Field, syn::token::Comma>> {
    let Fields::Named(fields) = &item_struct.fields else {
        return Err(Error::new_spanned(item_struct, message));
    };
    Ok(&fields.named)
}

fn parse_hook_path(
    attr: &Attribute,
    key: &str,
    module_ident: &Ident,
    span: &impl ToTokens,
) -> Result<Option<syn::Path>> {
    let Some(raw) = parse_attribute_string(attr, key) else {
        return Ok(None);
    };
    let mut path: syn::Path = syn::parse_str(&raw)
        .map_err(|_| Error::new_spanned(span, format!("{key} must be a valid Rust path")))?;
    if path.leading_colon.is_none() && path.segments.len() == 1 {
        path = syn::parse_quote! { #module_ident::#path };
    }
    Ok(Some(path))
}

fn validate_collections(
    solution: &SolutionMetadata,
    entities: &BTreeMap<String, EntityMetadata>,
    facts: &BTreeSet<String>,
    aliases: &BTreeMap<String, String>,
) -> Result<()> {
    for collection in &solution.collections {
        let resolved_type_name = canonical_type_name(aliases, &collection.type_name);
        if collection.descriptor_index.is_some() {
            if !entities.contains_key(resolved_type_name) {
                return Err(Error::new_spanned(
                    &collection.field_ident,
                    format!(
                        "planning_model! entity collection `{}` references unknown #[planning_entity] type `{}`",
                        collection.field_name, collection.type_name,
                    ),
                ));
            }
        } else if !facts.contains(resolved_type_name) && !entities.contains_key(resolved_type_name)
        {
            return Err(Error::new_spanned(
                &collection.field_ident,
                format!(
                    "planning_model! problem fact collection `{}` references unknown #[problem_fact] type `{}`",
                    collection.field_name, collection.type_name,
                ),
            ));
        }
    }
    Ok(())
}

fn validate_list_element_sources(
    solution: &SolutionMetadata,
    entities: &BTreeMap<String, EntityMetadata>,
    aliases: &BTreeMap<String, String>,
) -> Result<()> {
    for collection in solution
        .collections
        .iter()
        .filter(|collection| collection.descriptor_index.is_some())
    {
        let resolved_type_name = canonical_type_name(aliases, &collection.type_name);
        let Some(entity) = entities.get(resolved_type_name) else {
            continue;
        };
        let Some(element_collection) = entity.list_element_collection.as_deref() else {
            continue;
        };
        if !solution.collection_field_names.contains(element_collection) {
            return Err(Error::new_spanned(
                &collection.field_ident,
                format!(
                    "planning_model! list entity `{}` requires a solution collection field named `{}`",
                    entity.type_name, element_collection,
                ),
            ));
        }
    }
    Ok(())
}

fn collection_type_name(ty: &Type) -> Option<String> {
    let inner = collection_inner_type(ty)?;
    type_name(inner)
}

fn collection_inner_type(ty: &Type) -> Option<&Type> {
    let Type::Path(type_path) = ty else {
        return None;
    };
    let segment = type_path.path.segments.last()?;
    if segment.ident != "Vec" {
        return None;
    }
    let syn::PathArguments::AngleBracketed(args) = &segment.arguments else {
        return None;
    };
    let Some(syn::GenericArgument::Type(inner)) = args.args.first() else {
        return None;
    };
    Some(inner)
}

fn type_name(ty: &Type) -> Option<String> {
    let Type::Path(type_path) = ty else {
        return None;
    };
    Some(type_path.path.segments.last()?.ident.to_string())
}

fn field_is_option_usize(ty: &Type) -> bool {
    let Type::Path(type_path) = ty else {
        return false;
    };
    let Some(segment) = type_path.path.segments.last() else {
        return false;
    };
    if segment.ident != "Option" {
        return false;
    }
    let syn::PathArguments::AngleBracketed(args) = &segment.arguments else {
        return false;
    };
    let Some(syn::GenericArgument::Type(Type::Path(inner))) = args.args.first() else {
        return false;
    };
    inner
        .path
        .segments
        .last()
        .is_some_and(|segment| segment.ident == "usize")
}

fn generate_support_impl(model: &ModelMetadata) -> Result<TokenStream> {
    let solution_module = &model.solution.module_ident;
    let solution_ident = &model.solution.ident;
    let solution_path = quote! { #solution_module::#solution_ident };

    let mut descriptor_helpers = Vec::new();
    let mut descriptor_attachments = Vec::new();
    let mut runtime_helpers = Vec::new();
    let mut runtime_attachments = Vec::new();
    let mut validation_checks = Vec::new();
    let shadow_methods = generate_shadow_methods(model)?;

    for collection in model
        .solution
        .collections
        .iter()
        .filter(|collection| collection.descriptor_index.is_some())
    {
        let entity = model
            .entities
            .get(canonical_type_name(&model.aliases, &collection.type_name))
            .expect("entity collection should have been validated");
        let descriptor_index = collection.descriptor_index.unwrap();
        let entity_field = &collection.field_ident;
        let entity_accessor = format_ident!("__solverforge_entity_{}", entity_field);
        let entity_type_name = &entity.type_name;
        let solution_field_name = &collection.field_name;

        validation_checks.push(quote! {
            {
                let entity_descriptor = descriptor
                    .entity_descriptors
                    .get(#descriptor_index)
                    .expect("planning_model! entity descriptor missing");
                assert_eq!(
                    entity_descriptor.solution_field,
                    #solution_field_name,
                    "planning_model! entity descriptor field mismatch",
                );
                assert_eq!(
                    entity_descriptor.type_name,
                    #entity_type_name,
                    "planning_model! entity descriptor type mismatch",
                );
            }
        });

        for variable in &entity.scalar_variables {
            let variable_name = &variable.field_name;
            validation_checks.push(quote! {
                {
                    let entity_descriptor = descriptor
                        .entity_descriptors
                        .get(#descriptor_index)
                        .expect("planning_model! entity descriptor missing");
                    let _ = entity_descriptor
                        .variable_descriptors
                        .iter()
                        .find(|variable| {
                            variable.name == #variable_name
                                && variable.usize_getter.is_some()
                                && variable.usize_setter.is_some()
                        })
                        .expect("planning_model! scalar variable descriptor missing");
                }
            });

            if let Some(path) = &variable.hooks.nearby_value_distance_meter {
                let helper = format_ident!(
                    "__solverforge_descriptor_nearby_value_distance_{}_{}",
                    entity_field,
                    variable.field_name
                );
                let runtime_helper = format_ident!(
                    "__solverforge_runtime_nearby_value_distance_{}_{}",
                    entity_field,
                    variable.field_name
                );
                descriptor_helpers.push(quote! {
                    fn #helper(
                        solution: &dyn ::std::any::Any,
                        entity_index: usize,
                        value: usize,
                    ) -> f64 {
                        let solution = solution
                            .downcast_ref::<#solution_path>()
                            .expect("solution type mismatch for nearby value distance meter");
                        let entity = #solution_path::#entity_accessor(solution, entity_index);
                        #path(solution, entity, value)
                    }
                });
                descriptor_attachments.push(quote! {
                    attach_scalar_variable_hook(
                        descriptor,
                        #descriptor_index,
                        #variable_name,
                        |variable| {
                            variable.nearby_value_distance_meter = ::core::option::Option::Some(#helper);
                        },
                    );
                });
                runtime_helpers.push(quote! {
                    fn #runtime_helper(
                        solution: &#solution_path,
                        entity_index: usize,
                        _variable_index: usize,
                        value: usize,
                    ) -> ::core::option::Option<f64> {
                        let entity = #solution_path::#entity_accessor(solution, entity_index);
                        ::core::option::Option::Some(#path(solution, entity, value))
                    }
                });
                runtime_attachments.push(quote! {
                    if context.descriptor_index == #descriptor_index
                        && context.variable_name == #variable_name
                    {
                        context = context.with_nearby_value_distance_meter(#runtime_helper);
                    }
                });
            }

            if let Some(path) = &variable.hooks.nearby_entity_distance_meter {
                let helper = format_ident!(
                    "__solverforge_descriptor_nearby_entity_distance_{}_{}",
                    entity_field,
                    variable.field_name
                );
                let runtime_helper = format_ident!(
                    "__solverforge_runtime_nearby_entity_distance_{}_{}",
                    entity_field,
                    variable.field_name
                );
                descriptor_helpers.push(quote! {
                    fn #helper(
                        solution: &dyn ::std::any::Any,
                        left_entity_index: usize,
                        right_entity_index: usize,
                    ) -> f64 {
                        let solution = solution
                            .downcast_ref::<#solution_path>()
                            .expect("solution type mismatch for nearby entity distance meter");
                        let left = #solution_path::#entity_accessor(solution, left_entity_index);
                        let right = #solution_path::#entity_accessor(solution, right_entity_index);
                        #path(solution, left, right)
                    }
                });
                descriptor_attachments.push(quote! {
                    attach_scalar_variable_hook(
                        descriptor,
                        #descriptor_index,
                        #variable_name,
                        |variable| {
                            variable.nearby_entity_distance_meter = ::core::option::Option::Some(#helper);
                        },
                    );
                });
                runtime_helpers.push(quote! {
                    fn #runtime_helper(
                        solution: &#solution_path,
                        left_entity_index: usize,
                        right_entity_index: usize,
                        _variable_index: usize,
                    ) -> ::core::option::Option<f64> {
                        let left = #solution_path::#entity_accessor(solution, left_entity_index);
                        let right = #solution_path::#entity_accessor(solution, right_entity_index);
                        ::core::option::Option::Some(#path(solution, left, right))
                    }
                });
                runtime_attachments.push(quote! {
                    if context.descriptor_index == #descriptor_index
                        && context.variable_name == #variable_name
                    {
                        context = context.with_nearby_entity_distance_meter(#runtime_helper);
                    }
                });
            }

            if let Some(path) = &variable.hooks.construction_entity_order_key {
                let helper = format_ident!(
                    "__solverforge_descriptor_construction_entity_order_key_{}_{}",
                    entity_field,
                    variable.field_name
                );
                let runtime_helper = format_ident!(
                    "__solverforge_runtime_construction_entity_order_key_{}_{}",
                    entity_field,
                    variable.field_name
                );
                descriptor_helpers.push(quote! {
                    fn #helper(
                        solution: &dyn ::std::any::Any,
                        entity_index: usize,
                    ) -> i64 {
                        let solution = solution
                            .downcast_ref::<#solution_path>()
                            .expect("solution type mismatch for construction entity order key");
                        let entity = #solution_path::#entity_accessor(solution, entity_index);
                        #path(solution, entity)
                    }
                });
                descriptor_attachments.push(quote! {
                    attach_scalar_variable_hook(
                        descriptor,
                        #descriptor_index,
                        #variable_name,
                        |variable| {
                            variable.construction_entity_order_key = ::core::option::Option::Some(#helper);
                        },
                    );
                });
                runtime_helpers.push(quote! {
                    fn #runtime_helper(
                        solution: &#solution_path,
                        entity_index: usize,
                        _variable_index: usize,
                    ) -> ::core::option::Option<i64> {
                        let entity = #solution_path::#entity_accessor(solution, entity_index);
                        ::core::option::Option::Some(#path(solution, entity))
                    }
                });
                runtime_attachments.push(quote! {
                    if context.descriptor_index == #descriptor_index
                        && context.variable_name == #variable_name
                    {
                        context = context.with_construction_entity_order_key(#runtime_helper);
                    }
                });
            }

            if let Some(path) = &variable.hooks.construction_value_order_key {
                let helper = format_ident!(
                    "__solverforge_descriptor_construction_value_order_key_{}_{}",
                    entity_field,
                    variable.field_name
                );
                let runtime_helper = format_ident!(
                    "__solverforge_runtime_construction_value_order_key_{}_{}",
                    entity_field,
                    variable.field_name
                );
                descriptor_helpers.push(quote! {
                    fn #helper(
                        solution: &dyn ::std::any::Any,
                        entity_index: usize,
                        value: usize,
                    ) -> i64 {
                        let solution = solution
                            .downcast_ref::<#solution_path>()
                            .expect("solution type mismatch for construction value order key");
                        let entity = #solution_path::#entity_accessor(solution, entity_index);
                        #path(solution, entity, value)
                    }
                });
                descriptor_attachments.push(quote! {
                    attach_scalar_variable_hook(
                        descriptor,
                        #descriptor_index,
                        #variable_name,
                        |variable| {
                            variable.construction_value_order_key = ::core::option::Option::Some(#helper);
                        },
                    );
                });
                runtime_helpers.push(quote! {
                    fn #runtime_helper(
                        solution: &#solution_path,
                        entity_index: usize,
                        _variable_index: usize,
                        value: usize,
                    ) -> ::core::option::Option<i64> {
                        let entity = #solution_path::#entity_accessor(solution, entity_index);
                        ::core::option::Option::Some(#path(solution, entity, value))
                    }
                });
                runtime_attachments.push(quote! {
                    if context.descriptor_index == #descriptor_index
                        && context.variable_name == #variable_name
                    {
                        context = context.with_construction_value_order_key(#runtime_helper);
                    }
                });
            }
        }
    }

    Ok(quote! {
        impl ::solverforge::__internal::PlanningModelSupport for #solution_path {
            fn attach_descriptor_scalar_hooks(
                descriptor: &mut ::solverforge::__internal::SolutionDescriptor,
            ) {
                fn attach_scalar_variable_hook(
                    descriptor: &mut ::solverforge::__internal::SolutionDescriptor,
                    descriptor_index: usize,
                    variable_name: &'static str,
                    attach: impl FnOnce(&mut ::solverforge::__internal::VariableDescriptor),
                ) {
                    let entity_descriptor = descriptor
                        .entity_descriptors
                        .get_mut(descriptor_index)
                        .expect("planning_model! entity descriptor missing for scalar hook");
                    let variable_descriptor = entity_descriptor
                        .variable_descriptors
                        .iter_mut()
                        .find(|variable| {
                            variable.name == variable_name
                                && variable.usize_getter.is_some()
                                && variable.usize_setter.is_some()
                        })
                        .expect("planning_model! scalar hook target variable missing");
                    attach(variable_descriptor);
                }

                #(#descriptor_helpers)*
                #(#descriptor_attachments)*
            }

            fn attach_runtime_scalar_hooks(
                mut context: ::solverforge::__internal::ScalarVariableContext<Self>,
            ) -> ::solverforge::__internal::ScalarVariableContext<Self> {
                #(#runtime_helpers)*
                #(#runtime_attachments)*
                context
            }

            fn validate_model(descriptor: &::solverforge::__internal::SolutionDescriptor) {
                #(#validation_checks)*
            }

            #shadow_methods
        }
    })
}

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

#[cfg(test)]
mod tests {
    use quote::quote;

    use super::expand;

    #[test]
    fn expansion_tracks_every_manifest_module_as_include_dependency() {
        let expanded = expand(quote! {
            root = "tests/ui/pass/scalar_multi_module/domain";

            mod plan;
            mod task;
            mod worker;

            pub use plan::Plan;
            pub use task::Task;
            pub use worker::Worker;
        })
        .expect("planning_model! should expand")
        .to_string();

        assert!(expanded.contains("include_str !"));
        assert!(expanded.contains("plan.rs"));
        assert!(expanded.contains("task.rs"));
        assert!(expanded.contains("worker.rs"));
    }
}
