pub(super) fn field_is_option_usize(ty: &syn::Type) -> bool {
    field_option_inner_type(ty)
        .and_then(|inner| {
            let syn::Type::Path(inner_path) = inner else {
                return None;
            };
            inner_path.path.segments.last()
        })
        .map(|segment| segment.ident == "usize")
        .unwrap_or(false)
}

pub(super) fn field_option_inner_type(ty: &syn::Type) -> Option<&syn::Type> {
    let syn::Type::Path(type_path) = ty else {
        return None;
    };
    let segment = type_path.path.segments.last()?;
    if segment.ident != "Option" {
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
