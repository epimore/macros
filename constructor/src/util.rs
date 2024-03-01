pub fn to_snake_case(s: &str) -> String {
    let mut res = String::new();
    for c in s.chars() {
        if c.is_ascii_uppercase() {
            res.push('_');
            res.push(c.to_ascii_lowercase());
        } else {
            res.push(c)
        }
    }
    let mut pre_under_line = false;
    let mut result = String::new();
    for c in res.chars() {
        if c.eq(&'_') {
            if pre_under_line {
                continue;
            }
            pre_under_line = true;
        }else {
            pre_under_line = false;
        }
        result.push(c);

    }
    if result.starts_with('_') {
        result = result.split_off(1);
    }
    result
}
#[test]
fn test(){
    let a = "_abcde";
    let b = "_Abcde";
    let c = "_A_BcDe";
    let d = "A_Bcde";
    let e = "AbCde";
    println!("{}",to_snake_case(a));
    println!("{}",to_snake_case(b));
    println!("{}",to_snake_case(c));
    println!("{}",to_snake_case(d));
    println!("{}",to_snake_case(e));
}
