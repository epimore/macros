use mysql::serde::{Deserialize, Serialize};
use constructor::{Get, New, Set};
use ezsql::crud;

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Get, Set, New)]
#[new(device_id, domain_id, domain)]
#[crud(table_name = "GMV_OAUTH",
funs = [
{fn_name = "add_gmv_oauth", sql_type = "create:single", exist_update = "true"},
{fn_name = "add_gmv_oauth_by_batch", sql_type = "create:batch"},
{fn_name = "delete_gmv_oauth", sql_type = "delete", condition = "device_id:=,domain_id:="},
{fn_name = "update_gmv_oauth_pwd", sql_type = "update", fields = "pwd", condition = "device_id:=,domain_id:="},
{fn_name = "read_gmv_oauth_single_all", sql_type = "read:single", condition = "device_id:=,domain_id:="},
{fn_name = "read_gmv_oauth_single_pwd", sql_type = "read:single", fields = "pwd", condition = "device_id:=,domain_id:=", res_type = "false"},
{fn_name = "read_gmv_oauth_batch_status", sql_type = "read:batch", condition = "status:=", order = "device_id:desc" page = "true", res_type = "true"},
])]
struct GmvOauth {
    device_id: String,
    domain_id: String,
    domain: String,
    pwd: Option<String>,
    pwd_check: u8,
    alias: Option<String>,
    status: u8,
}

///The macro crud will generate the following code:
/// ************
/// impl User
/// {
/// pub fn add_gmv_oauth(& self, conn : & mut mysql :: PooledConn)
///   {
///       use mysql :: prelude :: Queryable ; use mysql :: params ; use common
///       :: err :: TransError ; use common :: log :: error ; let _ =
///       conn.exec_drop("INSERT INTO GMV_OAUTH (device_id,domain_id,domain,pwd,pwd_check,alias,status) VALUES (:device_id,:domain_id,:domain,:pwd,:pwd_check,:alias,:status) ON DUPLICATE KEY UPDATE device_id=VALUES(device_id),domain_id=VALUES(domain_id),domain=VALUES(domain),pwd=VALUES(pwd),pwd_check=VALUES(pwd_check),alias=VALUES(alias),status=VALUES(status)",
///       params!
///       {
///           "device_id" => & self.get_device_id(), "domain_id" => &
///           self.get_domain_id(), "domain" => & self.get_domain(), "pwd" => &
///           self.get_pwd(), "pwd_check" => & self.get_pwd_check(), "alias" =>
///           & self.get_alias(), "status" => & self.get_status()
///       }).hand_err(| msg | error! ("数据库操作失败: {msg}")) ;
///   } pub fn
///   add_gmv_oauth_by_batch(vec : Vec < Self >, conn : & mut mysql ::
///   PooledConn)
///   {
///       use mysql :: prelude :: Queryable ; use mysql :: params ; use common
///       :: err :: TransError ; use common :: log :: error ; let _ =
///       conn.exec_batch("INSERT INTO GMV_OAUTH (device_id,domain_id,domain,pwd,pwd_check,alias,status) VALUES (:device_id,:domain_id,:domain,:pwd,:pwd_check,:alias,:status)",
///       vec.iter().map(| p | params!
///       {
///           "device_id" => & p.get_device_id(), "domain_id" => &
///           p.get_domain_id(), "domain" => & p.get_domain(), "pwd" => &
///           p.get_pwd(), "pwd_check" => & p.get_pwd_check(), "alias" => &
///           p.get_alias(), "status" => & p.get_status()
///       })).hand_err(| msg | error! ("数据库操作失败: {msg}")) ;
///   } pub fn
///   delete_gmv_oauth(conn : & mut mysql :: PooledConn, device_id : String,
///   domain_id : String)
///   {
///       use mysql :: prelude :: Queryable ; use common :: err :: TransError ;
///       use mysql :: params ; use common :: log :: error ; let _ =
///       conn.exec_drop("DELETE FROM GMV_OAUTH WHERE device_id=:device_id AND domain_id=:domain_id",
///       params!
///       {
///           "device_id" => device_id, "domain_id" => domain_id
///       }).hand_err(| msg | error! ("数据库操作失败: {msg}")) ;
///   } pub fn
///   update_gmv_oauth_pwd(& self, conn : & mut mysql :: PooledConn, c_device_id
///   : String, c_domain_id : String)
///   {
///       use mysql :: prelude :: Queryable ; use common :: err :: TransError ;
///       use mysql :: params ; use common :: log :: error ; let _ =
///       conn.exec_drop("UPDATE GMV_OAUTH SET pwd=:pwd WHERE device_id=:c_device_id AND domain_id=:c_domain_id",
///       params!
///       {
///           "pwd" => & self.get_pwd(), c_device_id, c_domain_id
///       }).hand_err(| msg | error! ("数据库操作失败: {msg}")) ;
///   } pub fn
///   read_gmv_oauth_single_all(conn : & mut mysql :: PooledConn, device_id :
///   String, domain_id : String) -> common :: err :: GlobalResult < Option <
///   GmvOauth >>
///   {
///       use mysql :: prelude :: Queryable ; use common :: err :: TransError ;
///       use common :: log :: error ; use mysql :: params ; let res =
///       conn.exec_first("SELECT device_id,domain_id,domain,pwd,pwd_check,alias,status FROM GMV_OAUTH WHERE device_id=:device_id AND domain_id=:domain_id",
///       params!
///       {
///           device_id, domain_id
///       }).hand_err(| msg | error! ("数据库操作失败: {msg}"))
///       ?.map(| (device_id, domain_id, domain, pwd, pwd_check, alias, status)
///       | GmvOauth
///       { device_id, domain_id, domain, pwd, pwd_check, alias, status }) ;
///       Ok(res)
///   } pub fn read_gmv_oauth_single_pwd < T : mysql :: prelude :: FromRow >
///   (conn : & mut mysql :: PooledConn, device_id : String, domain_id : String)
///   -> common :: err :: GlobalResult < Option < T >>
///   {
///       use mysql :: prelude :: Queryable ; use common :: err :: TransError ;
///       use common :: log :: error ; use mysql :: params ; let res =
///       conn.exec_first("SELECT pwd FROM GMV_OAUTH WHERE device_id=:device_id AND domain_id=:domain_id",
///       params!
///       {
///           device_id, domain_id
///       }).hand_err(| msg | error! ("数据库操作失败: {msg}")) ? ;
///       Ok(res)
///   } pub fn
///   read_gmv_oauth_batch_status(conn : & mut mysql :: PooledConn, status : u8,
///   _limit_start : u32, _limit_end : u32) -> common :: err :: GlobalResult <
///   Vec < GmvOauth >>
///   {
///       use mysql :: prelude :: Queryable ; use common :: err :: TransError ;
///       use common :: log :: error ; use mysql :: params ; let res =
///       conn.exec("SELECT device_id,domain_id,domain,pwd,pwd_check,alias,status FROM GMV_OAUTH WHERE status=:status ORDER BY device_id DESC LIMIT :_limit_start, :_limit_end",
///       params!
///       {
///           status, _limit_start, _limit_end
///       }).hand_err(| msg | error! ("数据库操作失败: {msg}"))
///       ?.into_iter().map(|
///       (device_id, domain_id, domain, pwd, pwd_check, alias, status) |
///       GmvOauth
///       {
///           device_id, domain_id, domain, pwd, pwd_check, alias, status
///       }).collect() ; Ok(res)
///   }
/// }
/// ************


///配合https://github.com/epimore/pig使用
#[test]
fn test() {
}