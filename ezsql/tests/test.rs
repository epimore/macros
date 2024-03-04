use ezsql::crud;

#[derive(Default)]
#[crud(method = "update_user", fields = "name, age", condition = "id")]
#[crud(method = "read_user", fields = "name, age", condition = "id")]
#[crud(method = "delete_user",  condition = "id,age")]
#[crud(method = "create_user",  condition = "id,age")]
struct User {
    id: i32,
    name: String,
    age: i32,
    sex: bool,
}

#[test]
fn test(){
    // User::default()
}