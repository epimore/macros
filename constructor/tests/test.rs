use constructor::{Get, Set, New};

#[derive(Set, Get, New, Default, Debug)]
pub struct Foo {
    a: i32,
    b: String,
    c: bool,
}

#[test]
fn test_foo() {
    let foo = Foo::new(12i32, String::from("avac"), true);
    println!("{foo:?}");
    let x = Foo::default().set_a(112).set_b("bb".to_string());
    println!("{:?}", x);
    println!("{:?}", x.get_a());
    println!("{:?}", x.get_b());
    println!("{:?}", x.get_c());
}


#[derive(Set, Get, New, Default, Debug)]
#[set(a, c)]
#[get(b, d)]
#[new(b, c)]
pub struct Bar {
    a: i32,
    b: String,
    c: bool,
    d: f32,
}

#[test]
fn test_bar() {
    let bar = Bar::new("aaa".to_string(), true);
    println!("{bar:?}");
    let x = Bar::default().set_a(123);
    println!("{:?}", x);
    println!("{:?}", x.get_b());
    println!("{:?}", x.get_d());
}

#[derive(Set, Get, New, Default, Debug)]
pub struct UnFoo(u32, String, bool);

#[test]
fn test_un_foo() {
    let un_foo = UnFoo::new(1u32, "sss".to_string(), true);
    println!("{:?}", un_foo);
    let x = UnFoo::default().set_0(112u32).set_1("bb".to_string());
    println!("{:?}", x);
    println!("{:?}", x.get_0());
    println!("{:?}", x.get_1());
    println!("{:?}", x.get_2());
}


#[derive(Set, Get, New, Default, Debug)]
#[set(0, 2)]
#[get(1, 2, 3)]
#[new(0, 2)]
pub struct UnBar(u32, String, bool, i32);

#[test]
fn test_un_bar() {
    let un_bar = UnBar::new(234u32, true);
    println!("{un_bar:?}");
    let x = UnBar::default().set_0(123u32).set_2(true);
    println!("{:?}", x);
    println!("{:?}", x.get_1());
    println!("{:?}", x.get_2());
    println!("{:?}", x.get_3());
}

