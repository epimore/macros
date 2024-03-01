use constructor::{Get, Set};

#[derive(Set, Get, Default, Debug)]
pub struct Foo {
    pub a: i32,
    pub b: String,
    pub c: bool,
}

#[test]
fn test_foo() {
    let x = Foo::default().set_a(112).set_b("bb".to_string());
    println!("{:?}", x);
    println!("{:?}", x.get_a());
    println!("{:?}", x.get_b());
    println!("{:?}", x.get_c());
}

