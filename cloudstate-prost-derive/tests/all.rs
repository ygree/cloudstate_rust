
#[test]
fn tests() {
    let t = trybuild::TestCases::new();
    t.pass("tests/shopping_cart_test.rs");
    t.compile_fail("tests/missing_package_attr.rs");
    t.compile_fail("tests/incorrect_package_attribute.rs");
    t.compile_fail("tests/package_attribute_without_value.rs");
}