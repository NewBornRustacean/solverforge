use syn::{Error, Ident};

use crate::attr_parse::{get_attribute, has_attribute, parse_attribute_string};
use crate::list_registry::lookup_list_entity_metadata;

use super::type_helpers::extract_collection_inner_type;

pub(super) struct ListOwnerConfig<'a> {
    pub(super) field_ident: &'a Ident,
    pub(super) entity_type: &'a syn::Type,
    pub(super) element_collection_name: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ListElementCollectionKind {
    MatchingCollection,
    LegacyListCollection,
}

pub(super) struct ListElementCollectionConfig<'a> {
    pub(super) field_ident: &'a Ident,
    pub(super) owner_field: String,
    pub(super) kind: ListElementCollectionKind,
}

pub(super) struct ListRuntimeConfig<'a> {
    pub(super) list_owner: ListOwnerConfig<'a>,
    pub(super) element_collection: ListElementCollectionConfig<'a>,
}

fn type_name_from_collection(ty: &syn::Type) -> Option<String> {
    let entity_type = extract_collection_inner_type(ty)?;
    let syn::Type::Path(type_path) = entity_type else {
        return None;
    };
    let segment = type_path.path.segments.last()?;
    Some(segment.ident.to_string())
}

fn find_registered_list_owner_config<'a>(
    fields: &'a syn::punctuated::Punctuated<syn::Field, syn::token::Comma>,
) -> Result<Option<ListOwnerConfig<'a>>, Error> {
    let mut matches = Vec::new();

    for field in fields
        .iter()
        .filter(|f| has_attribute(&f.attrs, "planning_entity_collection"))
    {
        let Some(field_ident) = field.ident.as_ref() else {
            continue;
        };
        let Some(type_name) = type_name_from_collection(&field.ty) else {
            continue;
        };
        let Some(metadata) = lookup_list_entity_metadata(&type_name) else {
            continue;
        };
        let Some(entity_type) = extract_collection_inner_type(&field.ty) else {
            continue;
        };

        matches.push(ListOwnerConfig {
            field_ident,
            entity_type,
            element_collection_name: metadata.element_collection_name,
        });
    }

    if matches.len() > 1 {
        return Err(Error::new(
            proc_macro2::Span::call_site(),
            "#[planning_solution] currently supports at most one planning entity collection with #[planning_list_variable(...)]",
        ));
    }

    Ok(matches.pop())
}

pub(super) fn find_list_owner_config<'a>(
    config: &super::config::ShadowConfig,
    fields: &'a syn::punctuated::Punctuated<syn::Field, syn::token::Comma>,
) -> Result<Option<ListOwnerConfig<'a>>, Error> {
    let Some(list_owner) = config.list_owner.as_deref() else {
        return Ok(None);
    };

    fields
        .iter()
        .filter(|f| has_attribute(&f.attrs, "planning_entity_collection"))
        .find_map(|field| {
            let field_ident = field.ident.as_ref()?;
            if field_ident != list_owner {
                return None;
            }
            let entity_type = extract_collection_inner_type(&field.ty)?;
            let element_collection_name = type_name_from_collection(&field.ty)
                .and_then(|type_name| lookup_list_entity_metadata(&type_name))
                .map(|metadata| metadata.element_collection_name)
                .unwrap_or_default();
            Some(ListOwnerConfig {
                field_ident,
                entity_type,
                element_collection_name,
            })
        })
        .map(Some)
        .ok_or_else(|| {
            Error::new(
                proc_macro2::Span::call_site(),
                format!(
                    "#[shadow_variable_updates(list_owner = \"{list_owner}\")] must name a #[planning_entity_collection] field"
                ),
            )
        })
}

fn find_list_element_collection_config<'a>(
    fields: &'a syn::punctuated::Punctuated<syn::Field, syn::token::Comma>,
) -> Result<Option<ListElementCollectionConfig<'a>>, Error> {
    let mut matches = fields
        .iter()
        .filter_map(|field| {
            let attr = get_attribute(&field.attrs, "planning_list_element_collection")?;
            let owner = parse_attribute_string(attr, "owner")?;
            let field_ident = field.ident.as_ref()?;
            let inner = extract_collection_inner_type(&field.ty)?;
            let syn::Type::Path(type_path) = inner else {
                return None;
            };
            let segment = type_path.path.segments.last()?;
            if segment.ident != "usize" {
                return None;
            }
            Some(ListElementCollectionConfig {
                field_ident,
                owner_field: owner,
                kind: ListElementCollectionKind::LegacyListCollection,
            })
        })
        .collect::<Vec<_>>();

    if matches.len() > 1 {
        return Err(Error::new(
            proc_macro2::Span::call_site(),
            "#[planning_solution] currently supports at most one #[planning_list_element_collection(...)] field",
        ));
    }

    Ok(matches.pop())
}

pub(super) fn find_list_runtime_config<'a>(
    fields: &'a syn::punctuated::Punctuated<syn::Field, syn::token::Comma>,
) -> Result<Option<ListRuntimeConfig<'a>>, Error> {
    if let Some(list_owner) = find_registered_list_owner_config(fields)? {
        if let Some(element_collection) = fields.iter().find_map(|field| {
            let field_ident = field.ident.as_ref()?;
            if *field_ident != list_owner.element_collection_name {
                return None;
            }
            if has_attribute(&field.attrs, "planning_entity_collection")
                || has_attribute(&field.attrs, "problem_fact_collection")
            {
                Some(ListElementCollectionConfig {
                    field_ident,
                    owner_field: list_owner.field_ident.to_string(),
                    kind: ListElementCollectionKind::MatchingCollection,
                })
            } else {
                None
            }
        }) {
            return Ok(Some(ListRuntimeConfig {
                list_owner,
                element_collection,
            }));
        }
    }

    let Some(element_collection) = find_list_element_collection_config(fields)? else {
        if let Some(list_owner) = find_registered_list_owner_config(fields)? {
            return Err(Error::new(
                proc_macro2::Span::call_site(),
                format!(
                    "planning solution with list owner `{}` requires a `#[planning_entity_collection]` or `#[problem_fact_collection]` field named `{}`",
                    list_owner.field_ident,
                    list_owner.element_collection_name,
                ),
            ));
        }
        return Ok(None);
    };

    let owner = fields
        .iter()
        .filter(|f| has_attribute(&f.attrs, "planning_entity_collection"))
        .find_map(|field| {
            let field_ident = field.ident.as_ref()?;
            if *field_ident != element_collection.owner_field {
                return None;
            }
            let entity_type = extract_collection_inner_type(&field.ty)?;
            let element_collection_name = type_name_from_collection(&field.ty)
                .and_then(|type_name| lookup_list_entity_metadata(&type_name))
                .map(|metadata| metadata.element_collection_name)
                .unwrap_or_default();
            Some(ListOwnerConfig {
                field_ident,
                entity_type,
                element_collection_name,
            })
        })
        .ok_or_else(|| {
            Error::new(
                proc_macro2::Span::call_site(),
                format!(
                    "planning solution with list owner `{}` requires a `#[planning_list_element_collection(owner = \"{}\")]` field of type Vec<usize>",
                    element_collection.owner_field,
                    element_collection.owner_field,
                ),
            )
        })?;

    Ok(Some(ListRuntimeConfig {
        list_owner: owner,
        element_collection,
    }))
}

pub(super) fn shadow_updates_requested(config: &super::config::ShadowConfig) -> bool {
    config.inverse_field.is_some()
        || config.previous_field.is_some()
        || config.next_field.is_some()
        || config.cascading_listener.is_some()
        || config.post_update_listener.is_some()
        || !config.entity_aggregates.is_empty()
        || !config.entity_computes.is_empty()
}
