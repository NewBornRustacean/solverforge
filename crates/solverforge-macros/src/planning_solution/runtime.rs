use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Ident;

use super::type_helpers::extract_collection_inner_type;
use crate::attr_parse::has_attribute;

include!("runtime/solve.rs");
include!("runtime/scalar_setup.rs");
include!("runtime/helpers.rs");
