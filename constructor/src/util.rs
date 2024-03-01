pub fn to_snake_case(s: &str) -> String {
    let mut res = String::new();
    let mut pre_under_line = false;

    for c in s.chars() {
        if c.is_ascii_uppercase() {
            if !pre_under_line {
                res.push('_');
            }
            res.push(c.to_ascii_lowercase());
        } else {
            if c.eq(&'_') {
                pre_under_line = true;
            } else {
                pre_under_line = false;
            }
            res.push(c)
        }
    }
    if res.starts_with('_') {
        res = res.split_off(1);
    }
    res
}

#[test]
fn test() {
    assert_eq!(to_snake_case("_abcde"), String::from("abcde"));
    assert_eq!(to_snake_case("_Abcde"), String::from("abcde"));
    assert_eq!(to_snake_case("_A_BcDe"), String::from("a_bc_de"));
    assert_eq!(to_snake_case("A_Bcde"), String::from("a_bcde"));
    assert_eq!(to_snake_case("A_bCde"), String::from("a_b_cde"));
    assert_eq!(to_snake_case("AbCde"), String::from("ab_cde"));
    assert_eq!(to_snake_case("A1bCde"), String::from("a1b_cde"));
    assert_eq!(to_snake_case("Ab2Cde"), String::from("ab2_cde"));
}
