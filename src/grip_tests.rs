use crate::grip::magic_formula_response;

#[test]
fn magic_formula_response_is_zero_at_zero_slip() {
    let response = magic_formula_response(0.0, 10.5, 1.72, 0.32);
    assert!(response.abs() < 1e-6);
}

#[test]
fn magic_formula_response_grows_then_saturates() {
    let small = magic_formula_response(0.05, 10.5, 1.72, 0.32).abs();
    let medium = magic_formula_response(0.15, 10.5, 1.72, 0.32).abs();
    let large = magic_formula_response(0.45, 10.5, 1.72, 0.32).abs();

    assert!(medium > small);
    assert!(large <= 1.1);
}
