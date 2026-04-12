use super::*;

#[test]
fn test_simple_bounder_returns_none() {
    let bounder = SoftScoreBounder::new();
    assert!(format!("{:?}", bounder).contains("SoftScoreBounder"));
}

#[test]
fn test_bounder_type_display() {
    assert_eq!(format!("{}", BounderType::None), "None");
    assert_eq!(format!("{}", BounderType::Simple), "Simple");
    assert_eq!(format!("{}", BounderType::FixedOffset), "FixedOffset");
}

#[test]
fn test_bounder_type_default() {
    assert_eq!(BounderType::default(), BounderType::None);
}
