mod util;

use proc_macro::TokenStream;
use std::collections::HashMap;
use std::fmt::{self, Debug, Display, format};
use proc_macro2::{Group, Ident, Literal, TokenTree};
use quote::{format_ident, quote, ToTokens};
use syn::{parse_macro_input, Attribute, parse, Type};
use syn::DeriveInput;
use syn::parse::Parser;
use syn::spanned::Spanned;
use constructor::Set;
use tools::*;
use util::*;


#[proc_macro_attribute]
pub fn crud(attr: TokenStream, item: TokenStream) -> TokenStream {
    println!("begin ++++++");
    println!("attr: \"{}\"", &attr.to_string());
    println!("item: \"{}\"", &item.to_string());
    println!("*************");
    let attr_state = parse_attr(attr.clone());
    println!("------------  {attr_state:?}");
    println!("*************");
    println!("*************");
    let stream = item.clone();
    let item_input = parse_macro_input!(stream as DeriveInput);
    let struct_info = parse_item(item_input.clone());
    println!("-------{:?}", struct_info);
    println!("*************");
    println!("over ++++++\n\n");

    let constructor = build_constructor(attr, item);

    println!("+++++++ fn----> \n\n{}", &constructor.to_string());
    constructor.into()
}

fn build_constructor(attr: TokenStream, item: TokenStream) -> proc_macro2::TokenStream {
    let state = parse_attr(attr);
    let input: DeriveInput = syn::parse(item).expect("syn parse failed");
    let struct_info = parse_item(input.clone());
    let struct_name = &input.ident;
    let struct_table_field_map = build_struct_to_table_field_map(&state, &struct_info);
    let mut constructors = quote!();
    for inner in state.get_funs() {
        let (sql_type, ext) = inner.get_sql_type();
        let constructor = match sql_type {
            SqlType::CREATE => {
                build_create_constructor(inner, ext, state.get_table_name(), &struct_table_field_map)
            }
            SqlType::READ => { panic!() }
            SqlType::UPDATE => { build_update_constructor(inner, ext, state.get_table_name(), &struct_table_field_map) }
            SqlType::DELETE => { buidl_delete_constructor(inner, struct_info.get_field_infos(), state.get_table_name(), &struct_table_field_map) }
        };
        constructors = quote! {
            #constructors
            #constructor
        };
    }
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    quote! {
        #input
        impl #impl_generics #struct_name #ty_generics #where_clause {
             #constructors
        }
    }
}

fn build_update_constructor(inner: &Inner, sql_type_ext: &SqlTypeExt, table_name: &String, struct_table_field_map: &HashMap<String, String>) -> proc_macro2::TokenStream {
    let mut sql = String::from("INSERT INTO ");
    sql.push_str(table_name);
    sql.push_str(" (");
    let (table_field_name, struct_field_name, update_fields_create_str, struct_field_name_vec) = build_create_update_fields_str(&struct_table_field_map, inner.get_fields());
    sql.push_str(&*table_field_name);
    sql.push_str(") VALUES (");
    sql.push_str(&*struct_field_name);
    sql.push(')');
    if let Some(true) = inner.get_create() {
        sql.push_str(" ON DUPLICATE KEY UPDATE ");
        sql.push_str(&*update_fields_create_str);
    }
    let fn_name = inner.get_fn_name();
    let fn_name_ident = format_ident!("{fn_name}");
    let field_value_ident = struct_field_name_vec
        .iter()
        .map(|name| format_ident!("get_{}",name)).collect::<Vec<Ident>>();
    match *sql_type_ext {
        SqlTypeExt::SINGLE => {
            quote!(
                pub fn #fn_name_ident(&self, conn: &mut pig::mysql::PooledConn) {
                    use pig::mysql::prelude::Queryable;
                    use pig::mysql::params;
                    use pig::err::TransError;
                    use pig::logger::error;
                    let _ = conn.exec_drop(#sql,params!{
                        #(#struct_field_name_vec => &self.#field_value_ident()),*
                    }).hand_err(|msg| error!("数据库操作失败: {msg}"));
                }
            )
        }
        SqlTypeExt::BATCH => {
            quote!(
                pub fn #fn_name_ident(vec:Vec<Self>, conn: &mut pig::mysql::PooledConn) {
                    use pig::mysql::prelude::Queryable;
                    use pig::mysql::params;
                    use pig::err::TransError;
                    use pig::logger::error;
                    let _ = conn.exec_batch(#sql,vec.iter().map(|p|params!{
                       #(#struct_field_name_vec => &p.#field_value_ident()),*
                    })).hand_err(|msg| error!("数据库操作失败: {msg}"));
                }
            )
        }
    }
}

fn buidl_delete_constructor(inner: &Inner, field_infos: &HashMap<String, Type>, table_name: &String, struct_table_field_map: &HashMap<String, String>) -> proc_macro2::TokenStream {
    let mut sql = String::from("DELETE FROM ");
    sql.push_str(table_name);
    let fn_name = inner.get_fn_name();
    let fn_name_ident = format_ident!("{fn_name}");
    match inner.get_condition() {
        None => {
            quote!(
                pub fn #fn_name_ident(conn: &mut pig::mysql::PooledConn){
                    use pig::mysql::prelude::Queryable;
                    use pig::err::TransError;
                    use pig::logger::error;
                    let _ = conn.query_drop(#sql).hand_err(|msg| error!("数据库操作失败: {msg}"));
                }
            )
        }
        Some(map) => {
            sql.push_str(" WHERE ");
            let mut params = Vec::new();
            let mut field_names = Vec::new();
            let mut param_types = Vec::new();
            for (index, (field_name, condition)) in map.iter().enumerate() {
                if index > 0 {
                    sql.push_str(" AND ")
                }
                let table_field_name = struct_table_field_map.get(&*field_name).expect(&*format!("fn = {},condition field {} is invalid", &fn_name, &field_name));
                sql.push_str(&*table_field_name);
                sql.push_str(condition.get_value());
                field_names.push(field_name.clone());
                sql.push_str(&*format!(":{}", &field_name));
                params.push(format_ident!("{}",&field_name));
                param_types.push(field_infos.get(&*field_name).unwrap().clone());
            }
            quote!(
                pub fn #fn_name_ident(conn: &mut pig::mysql::PooledConn,#(#params:#param_types),*){
                    use pig::mysql::prelude::Queryable;
                    use pig::err::TransError;
                    use pig::mysql::params;
                    use pig::logger::error;
                    let _ = conn.exec_drop(#sql,params!{
                        #(#field_names => #params),*
                    }).hand_err(|msg| error!("数据库操作失败: {msg}"));
                }
            )
        }
    }
}

fn build_create_constructor(inner: &Inner, sql_type_ext: &SqlTypeExt, table_name: &String, struct_table_field_map: &HashMap<String, String>) -> proc_macro2::TokenStream {
    let mut sql = String::from("INSERT INTO ");
    sql.push_str(table_name);
    sql.push_str(" (");
    let (table_field_name, struct_field_name, _update_fields_create_str, struct_field_name_vec) = build_create_update_fields_str(&struct_table_field_map, inner.get_fields());
    sql.push_str(&*table_field_name);
    sql.push_str(") VALUES (");
    sql.push_str(&*struct_field_name);
    sql.push(')');
    let fn_name = inner.get_fn_name();
    let fn_name_ident = format_ident!("{fn_name}");
    let field_value_ident = struct_field_name_vec
        .iter()
        .map(|name| format_ident!("get_{}",name)).collect::<Vec<Ident>>();
    match *sql_type_ext {
        SqlTypeExt::SINGLE => {
            quote!(
                pub fn #fn_name_ident(&self, conn: &mut pig::mysql::PooledConn) {
                    use pig::mysql::prelude::Queryable;
                    use pig::mysql::params;
                    use pig::err::TransError;
                    use pig::logger::error;
                    let _ = conn.exec_drop(#sql,params!{
                        #(#struct_field_name_vec => &self.#field_value_ident()),*
                    }).hand_err(|msg| error!("数据库操作失败: {msg}"));
                }
            )
        }
        SqlTypeExt::BATCH => {
            quote!(
                pub fn #fn_name_ident(vec:Vec<Self>, conn: &mut pig::mysql::PooledConn) {
                    use pig::mysql::prelude::Queryable;
                    use pig::mysql::params;
                    use pig::err::TransError;
                    use pig::logger::error;
                    let _ = conn.exec_batch(#sql,vec.iter().map(|p|params!{
                       #(#struct_field_name_vec => &p.#field_value_ident()),*
                    })).hand_err(|msg| error!("数据库操作失败: {msg}"));
                }
            )
        }
    }
}

/// res_field_map -> struct_field_name:table_field_name
/// fields -> 指定field
/// 未指定field时，全部字段
/// table_fields_str,struct_fields_str,update_fields_create_str,struct_fields_vec
fn build_create_update_fields_str(struct_table_field_map: &HashMap<String, String>, fields: &Option<Vec<String>>) -> (String, String, String, Vec<String>) {
    let mut table_fields_str = String::new();
    let mut struct_fields_str = String::new();
    let mut update_fields_create_str = String::new();
    let mut struct_fields_vec = Vec::new();
    match fields {
        None => {
            for (inedx, (key, val)) in struct_table_field_map.iter().enumerate() {
                if inedx > 0 {
                    table_fields_str.push(',');
                    struct_fields_str.push(',');
                    update_fields_create_str.push(',');
                }
                table_fields_str.push_str(&*val);
                update_fields_create_str.push_str(&*format!("{}=VALUES({})", &val, &val));
                struct_fields_str.push_str(&*format!(":{}", key));
                struct_fields_vec.push(key.clone());
            }
        }
        Some(vec) => {
            for (index, key) in vec.iter().enumerate() {
                if index > 0 {
                    table_fields_str.push(',');
                    struct_fields_str.push(',');
                    update_fields_create_str.push(',');
                }
                let table_field = struct_table_field_map.get(key).expect(&*format!("{} invalid fields", key));
                table_fields_str.push_str(&*table_field);
                update_fields_create_str.push_str(&*format!("{}=VALUES({})", &table_field, &table_field));
                struct_fields_str.push_str(&*format!(":{}", key));
            }
            struct_fields_vec = vec.clone();
        }
    }
    (table_fields_str, struct_fields_str, update_fields_create_str, struct_fields_vec)
}

//全量struct字段映射table字段：field_name_to_snake = true -> 全局结构体字段除alias_fields不受影响，其他转为sanke结构；
///返回struct_field_name:table_field_name
fn build_struct_to_table_field_map(attr_state: &State, struct_info: &StructInfo) -> HashMap<String, String> {
    let mut res_map: HashMap<String, String> = HashMap::new();
    if let Some(map) = attr_state.get_alias_fields() {
        if let Some(true) = attr_state.get_field_name_to_snake() {
            for (struct_field_name, _field_type) in struct_info.get_field_infos() {
                let table_field_name = map.get(struct_field_name).map(|str| str.clone()).unwrap_or_else(|| to_snake_case(&*struct_field_name));
                res_map.insert(struct_field_name.clone(), table_field_name);
            }
        } else {
            for (struct_field_name, _field_type) in struct_info.get_field_infos() {
                let table_field_name = map.get(struct_field_name).map(|str| str.clone()).unwrap_or(struct_field_name.clone());
                res_map.insert(struct_field_name.clone(), table_field_name);
            }
        }
    }
    res_map
}
