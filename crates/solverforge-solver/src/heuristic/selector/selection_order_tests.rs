use super::*;

#[test]
fn test_resolve_inherit_from_random() {
    let order = SelectionOrder::Inherit;
    assert_eq!(
        order.resolve(SelectionOrder::Random),
        SelectionOrder::Random
    );
}

#[test]
fn test_resolve_inherit_from_original() {
    let order = SelectionOrder::Inherit;
    assert_eq!(
        order.resolve(SelectionOrder::Original),
        SelectionOrder::Original
    );
}

#[test]
fn test_resolve_inherit_from_inherit() {
    let order = SelectionOrder::Inherit;
    assert_eq!(
        order.resolve(SelectionOrder::Inherit),
        SelectionOrder::Random
    );
}

#[test]
fn test_resolve_non_inherit() {
    let order = SelectionOrder::Original;
    assert_eq!(
        order.resolve(SelectionOrder::Random),
        SelectionOrder::Original
    );
}

#[test]
fn test_is_random() {
    assert!(SelectionOrder::Random.is_random());
    assert!(SelectionOrder::Shuffled.is_random());
    assert!(SelectionOrder::Probabilistic.is_random());

    assert!(!SelectionOrder::Original.is_random());
    assert!(!SelectionOrder::Sorted.is_random());
    assert!(!SelectionOrder::Inherit.is_random());
}

#[test]
fn test_requires_caching() {
    assert!(SelectionOrder::Shuffled.requires_caching());
    assert!(SelectionOrder::Sorted.requires_caching());
    assert!(SelectionOrder::Probabilistic.requires_caching());

    assert!(!SelectionOrder::Original.requires_caching());
    assert!(!SelectionOrder::Random.requires_caching());
    assert!(!SelectionOrder::Inherit.requires_caching());
}

#[test]
fn test_from_random_selection() {
    assert_eq!(
        SelectionOrder::from_random_selection(true),
        SelectionOrder::Random
    );
    assert_eq!(
        SelectionOrder::from_random_selection(false),
        SelectionOrder::Original
    );
}

#[test]
fn test_to_random_selection() {
    assert!(SelectionOrder::Random.to_random_selection());
    assert!(!SelectionOrder::Original.to_random_selection());
}

#[test]
#[should_panic(expected = "cannot be converted")]
fn test_to_random_selection_panics_on_shuffled() {
    SelectionOrder::Shuffled.to_random_selection();
}

#[test]
fn test_default() {
    assert_eq!(SelectionOrder::default(), SelectionOrder::Inherit);
}
