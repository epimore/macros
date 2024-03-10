pub const CREATE: &str = "CREATE";
pub const READ: &str = "READ";
pub const UPDATE: &str = "UPDATE";
pub const DELETE: &str = "DELETE";
pub const ASC: &str = "ASC";
pub const DESC: &str = "DESC";
pub const SINGLE: &str = "SINGLE";
pub const BATCH: &str = "BATCH";
pub const EQ: &str = "=";
pub const LT: &str = "<";
pub const GT: &str = ">";
pub const EQ_LT: &str = "<=";
pub const EQ_GT: &str = ">=";
pub const NEQ: &str = "!=";

pub enum Condition {
    EQ,
    LT,
    GT,
    EqLt,
    EqGt,
    NEQ,
}

impl Condition {
    pub fn match_type(str: &str) -> Self {
        match str {
            EQ => Condition::EQ,
            LT => Condition::LT,
            GT => Condition::GT,
            EQ_LT | "=<" => Condition::EqLt,
            EQ_GT | "=>" => Condition::EqGt,
            NEQ | "=>" => Condition::NEQ,
            &_ => { panic!("invalid Condition;should be one of [>,=,<,>=,=>,<=,=<,!=]") }
        }
    }

    pub fn get_value(&self) -> &str {
        match self {
            Condition::EQ => EQ,
            Condition::LT => LT,
            Condition::GT => GT,
            Condition::EqLt => EQ_LT,
            Condition::EqGt => EQ_GT,
            Condition::NEQ => NEQ,
        }
    }
}

pub enum SqlType {
    CREATE,
    READ,
    UPDATE,
    DELETE,
}

impl SqlType {
    pub fn match_type(str: &str) -> Self {
        match &*str.to_ascii_uppercase() {
            CREATE => { SqlType::CREATE }
            READ => { SqlType::READ }
            UPDATE => { SqlType::UPDATE }
            DELETE => { SqlType::DELETE }
            _ => { panic!("invalid SqlType;should be one of [CREATE,READ,UPDATE,DELETE]") }
        }
    }
    pub fn get_value(&self) -> &str {
        match self {
            SqlType::CREATE => { CREATE }
            SqlType::READ => { READ }
            SqlType::UPDATE => { UPDATE }
            SqlType::DELETE => { DELETE }
        }
    }
}

pub enum SqlTypeExt {
    SINGLE,
    BATCH,
}

impl SqlTypeExt {
    pub fn match_type(str: &str) -> Self {
        match &*str.to_ascii_uppercase() {
            SINGLE => { SqlTypeExt::SINGLE }
            BATCH => { SqlTypeExt::BATCH }
            _ => { panic!("invalid SqlTypeExt;should be one of [SINGLE,BATCH]") }
        }
    }

    pub fn get_value(&self) -> &str {
        match self {
            SqlTypeExt::SINGLE => { SINGLE }
            SqlTypeExt::BATCH => { BATCH }
        }
    }
}

pub enum Order {
    ASC,
    DESC,
}

impl Order {
    pub fn match_type(str: &str) -> Self {
        match &*str.to_ascii_uppercase() {
            ASC => { Order::ASC }
            DESC => { Order::DESC }
            _ => { panic!("invalid SqlType;should be one of [ASC,DESC]") }
        }
    }
    pub fn get_value(&self) -> &str {
        match self {
            Order::ASC => { ASC }
            Order::DESC => { DESC }
        }
    }
}

// pub trait Trans<Input,Output>{
//     fn to_val(input:Input)-> Output;
// }


#[allow(dead_code)]
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
    //不允许下划线开始
    if res.starts_with('_') {
        res = res.split_off(1);
    }
    res
}

pub enum Private {
    CREATE,
    READ(),
    UPDATE,
    DELETE,

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

#[test]
fn test1() {
    let sql_type = SqlType::match_type("UPDATE");
    let x = sql_type.get_value();
    println!("{x}");
}