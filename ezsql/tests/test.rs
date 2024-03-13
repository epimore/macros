use constructor::{Get, New, Set};
use ezsql::crud;

#[derive(Debug, Default, Get, Set, New)]
#[crud(table_name = "xxx", alias_fields = "xx:xxx,aa:bb", field_name_to_snake = true,
funs = [
{fn_name = "create_user1", sql_type = "create:single", fields = "name, age"},
{fn_name = "create_user2", sql_type = "create:batch", fields = "name, age", exist_update = "true"},
{fn_name = "delete_user1", sql_type = "delete", condition = "name:=, age:<=,id:="}
{fn_name = "delete_user2", sql_type = "delete"},
{fn_name = "update_user1", sql_type = "update", fields = "name, age", condition = "name:>, age:=<,id:="},
{fn_name = "update_user2", sql_type = "update", fields = "name, age", condition = "name:>, age:=<,id:="},
{fn_name = "read_user1", sql_type = "read:single", pre_where_sql = "select count(1)"},
{fn_name = "read_user2", sql_type = "read:single", fields = "name, age", condition = "name:=, age:>=,id:=", order = "name:DESC,age:ASC", res_type = "false"},
{fn_name = "read_user3", sql_type = "read:batch", fields = "name, age", condition = "name:=, age:>=,id:=", page = "true", order = "name:DESC,age:ASC", res_type = "true"},
{fn_name = "read_user4", sql_type = "read:batch", fields = "name, age", condition = "name:=, age:>=,id:=", order = "name:DESC,age:ASC", res_type = "true"},
{fn_name = "read_user5", sql_type = "read:single", fields = "name, age", condition = "name:=, age:>=,id:=", order = "name:DESC,age:ASC", res_type = "true"},
{fn_name = "read_user6", sql_type = "read:single", condition = "name:=, age:>=,id:=", pre_where_sql = "select count(1)"}
])]
struct User {
    id: i32,
    name: String,
    age: i32,
    sex: bool,
}

///The macro crud will generate the following code:
/// ************
/// impl User
/// {
///     pub fn create_user1(&self, conn: &mut mysql::PooledConn)
///     {
///         use mysql::prelude::Queryable;
///         use mysql::params;
///         use common::err::TransError;
///         use common::log::error;
///         let _ =
///             conn.exec_drop("INSERT INTO xxx (name,age) VALUES (:name,:age)",
///                            params! {
///             "name" => & self.get_name(), "age" => & self.get_age()
///         }).hand_err(|msg| error!("数据库操作失败: {msg}"));
///     }
///     pub fn create_user2(vec: Vec<Self>, conn: &mut mysql::PooledConn)
///     {
///         use mysql::prelude::Queryable;
///         use mysql::params;
///         use common::err::TransError;
///         use common::log::error;
///         let _ =
///             conn.exec_batch("INSERT INTO xxx (name,age) VALUES (:name,:age) ON DUPLICATE KEY UPDATE name=VALUES(name),age=VALUES(age)",
///                             vec.iter().map(|p| params! {
///             "name" => & p.get_name(), "age" => & p.get_age()
///         })).hand_err(|msg| error!("数据库操作失败: {msg}"));
///     }
///     pub fn delete_user1(conn: &mut mysql::PooledConn, id: i32, age: i32, name: String)
///     {
///         use mysql::prelude::Queryable;
///         use common::err::TransError;
///         use mysql::params;
///         use common::log::error;
///         let _ =
///             conn.exec_drop("DELETE FROM xxx WHERE id=:id AND age<=:age AND name=:name",
///                            params! {
///             "id" => id, "age" => age, "name" => name
///         }).hand_err(|msg| error!("数据库操作失败: {msg}"));
///     }
///     pub fn delete_user2(conn: &mut mysql::PooledConn)
///     {
///         use mysql::prelude::Queryable;
///         use common::err::TransError;
///         use common::log::error;
///         let _ =
///             conn.query_drop("DELETE FROM xxx").hand_err(|msg| error!("数据库操作失败: {msg}"));
///     }
///     pub fn update_user1(&self, conn: &mut mysql::PooledConn, c_id: i32, c_age: i32, c_name: String)
///     {
///         use mysql::prelude::Queryable;
///         use common::err::TransError;
///         use mysql::params;
///         use common::log::error;
///         let _ =
///             conn.exec_drop("UPDATE xxx SET name=:name,age=:age WHERE id=:c_id AND age<=:c_age AND name>:c_name",
///                            params! {
///             "name" => & self.get_name(), "age" => & self.get_age(), c_id, c_age, c_name
///         }).hand_err(|msg| error!("数据库操作失败: {msg}"));
///     }
///     pub fn update_user2(&self, conn: &mut mysql::PooledConn, c_id: i32, c_name: String, c_age: i32)
///     {
///         use mysql::prelude::Queryable;
///         use common::err::TransError;
///         use mysql::params;
///         use common::log::error;
///         let _ =
///             conn.exec_drop("UPDATE xxx SET name=:name,age=:age WHERE id=:c_id AND name>:c_name AND age<=:c_age",
///                            params! {
///             "name" => & self.get_name(), "age" => & self.get_age(), c_id, c_name, c_age
///         }).hand_err(|msg| error!("数据库操作失败: {msg}"));
///     }
///     pub fn read_user1<T: mysql::prelude::FromRow>(&self, conn: &mut mysql::PooledConn) -> common::err::GlobalResult<Option<T>>
///     {
///         use mysql::prelude::Queryable;
///         use common::err::TransError;
///         use common::log::error;
///         let res =
///             conn.query_first("select count(1) FROM xxx").hand_err(|msg| error!("数据库操作失败: {msg}"))?;
///         Ok(res)
///     }
///     pub fn read_user2<T: mysql::prelude::FromRow>(&self, conn: &mut mysql::PooledConn, name: String, age: i32, id: i32)
///                                                   -> common::err::GlobalResult<Option<T>>
///     {
///         use mysql::prelude::Queryable;
///         use common::err::TransError;
///         use common::log::error;
///         use mysql::params;
///         let res =
///             conn.exec_first("SELECT name,age FROM xxx WHERE name=:name AND age>=:age AND id=:id ORDER BY age ASC,name DESC",
///                             params! {
///             name, age, id
///         }).hand_err(|msg| error!("数据库操作失败: {msg}"))?;
///         Ok(res)
///     }
///     pub fn read_user3(&self, conn: &mut mysql::PooledConn, _limit_start: u32, _limit_end: u32, name: String, age: i32, id: i32)
///                       -> common::err::GlobalResult<Vec<User>>
///     {
///         use mysql::prelude::Queryable;
///         use common::err::TransError;
///         use common::log::error;
///         use mysql::params;
///         let res =
///             conn.exec("SELECT name,age FROM xxx WHERE name=:name AND age>=:age AND id=:id ORDER BY name DESC,age ASC LIMIT :_limit_start, :_limit_end",
///                       params! {
///             name, age, id, _limit_start, _limit_end
///         }).hand_err(|msg| error!("数据库操作失败: {msg}"))?
///                 .into_iter().map(|(name, age)| User
///             { name, age, ..Default::default() }).collect();
///         Ok(res)
///     }
///     pub fn read_user4(&self, conn: &mut mysql::PooledConn, age: i32, name: String, id: i32) -> common::err::GlobalResult<Vec<User>>
///     {
///         use mysql::prelude::Queryable;
///         use common::err::TransError;
///         use common::log::error;
///         use mysql::params;
///         let res =
///             conn.exec("SELECT name,age FROM xxx WHERE age>=:age AND name=:name AND id=:id ORDER BY name DESC,age ASC",
///                       params! {
///             age, name, id
///         }).hand_err(|msg| error!("数据库操作失败: {msg}"))?
///                 .into_iter().map(|(name, age)| User { name, age, ..Default::default() })
///                 .collect();
///         Ok(res)
///     }
///     pub fn read_user5(&self, conn: &mut mysql::PooledConn, name: String, age: i32, id: i32) -> common::err::GlobalResult<Option<User>>
///     {
///         use mysql::prelude::Queryable;
///         use common::err::TransError;
///         use common::log::error;
///         use mysql::params;
///         let res =
///             conn.exec_first("SELECT name,age FROM xxx WHERE name=:name AND age>=:age AND id=:id ORDER BY age ASC,name DESC",
///                             params! {
///             name, age, id
///         }).hand_err(|msg| error!("数据库操作失败: {msg}"))?
///                 .map(|(name, age)| User { name, age, ..Default::default() });
///         Ok(res)
///     }
///     pub fn read_user6<T: mysql::prelude::FromRow>(&self, conn: &mut mysql::PooledConn, age: i32, name: String, id: i32)
///                                                   -> common::err::GlobalResult<Option<T>>
///     {
///         use mysql::prelude::Queryable;
///         use common::err::TransError;
///         use common::log::error;
///         use mysql::params;
///         let res =
///             conn.exec_first("select count(1) FROM xxx WHERE age>=:age AND name=:name AND id=:id",
///                             params! {
///             age, name, id
///         }).hand_err(|msg| error!("数据库操作失败: {msg}"))?;
///         Ok(res)
///     }
/// }
/// ************
#[test]
fn test() {
    // let user = User::new(1, "a".to_string(), 2, true);
    let user = User::default();
    println!("{user:?}");
}

