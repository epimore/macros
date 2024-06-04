mod util;

use proc_macro::TokenStream;
use proc_macro2::{Ident};
use quote::{format_ident, quote};
use syn::{Type};
use syn::DeriveInput;
use util::*;
use indexmap::IndexMap;

///结构体字段暂只支持snake_case命名
/// 结构体字段、库表字段、条件参数字段分开
#[proc_macro_attribute]
pub fn crud(attr: TokenStream, item: TokenStream) -> TokenStream {
    let constructor = build_constructor(attr, item);
    constructor.into()
}
///方法参数顺序：self,conn,condition,limit
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
            SqlType::READ => { build_read_constructor(inner, struct_name.clone(), ext, state.get_table_name(), &struct_table_field_map, struct_info.get_field_infos()) }
            SqlType::UPDATE => { build_update_constructor(inner, struct_info.get_field_infos(), ext, state.get_table_name(), &struct_table_field_map) }
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

///没有指定返回值类型时
fn build_read_constructor(inner: &Inner, struct_name: Ident, sql_type_ext: &SqlTypeExt, table_name: &String, struct_table_field_map: &IndexMap<String, String>, field_infos: &IndexMap<String, Type>) -> proc_macro2::TokenStream {
    let (sql, res_ident, condition_param_name_type) = build_read_sql(inner, table_name, struct_table_field_map, field_infos);
    let fn_name = inner.get_fn_name();
    let fn_name_ident = format_ident!("{fn_name}");
    match inner.get_page() {
        Some(true) => {
            match *sql_type_ext {
                SqlTypeExt::SINGLE => {
                    match condition_param_name_type {
                        None => {
                            match res_ident {
                                //指定了sql前缀,返回原始数据，需自己处理结果转换,res_type无效
                                None => {
                                    quote!(
                                    pub fn #fn_name_ident<T: mysql::prelude::FromRow>(conn: &mut mysql::PooledConn)->common::err::GlobalResult<Option<T>>{
                                        use mysql::prelude::Queryable;
                                        use common::err::TransError;
                                        use common::log::error;
                                        let res = conn.query_first(#sql).hand_log(|msg| error!("数据库操作失败: {msg}"))?;
                                        Ok(res)
                                    })
                                }
                                Some(vec) => {
                                    match inner.get_res_type() {
                                        Some(true) => {
                                            if vec.len() == field_infos.len() {
                                                quote!(
                                    pub fn #fn_name_ident(conn: &mut mysql::PooledConn)->common::err::GlobalResult<Option<#struct_name>>{
                                        use mysql::prelude::Queryable;
                                        use common::err::TransError;
                                        use common::log::error;
                                        let res = conn.query_first(#sql).hand_log(|msg| error!("数据库操作失败: {msg}"))?.map(|(#(#vec),*)|#struct_name{#(#vec),*});
                                            Ok(res)
                                    })
                                            } else {
                                                quote!(
                                    pub fn #fn_name_ident(conn: &mut mysql::PooledConn)->common::err::GlobalResult<Option<#struct_name>>{
                                        use mysql::prelude::Queryable;
                                        use common::err::TransError;
                                        use common::log::error;
                                        let res = conn.query_first(#sql).hand_log(|msg| error!("数据库操作失败: {msg}"))?.map(|(#(#vec),*)|#struct_name{#(#vec),*,..Default::default()});
                                            Ok(res)
                                    })
                                            }
                                        }
                                        _ => {
                                            quote!(
                                    pub fn #fn_name_ident<T: mysql::prelude::FromRow>(conn: &mut mysql::PooledConn)->common::err::GlobalResult<Option<T>>{
                                        use mysql::prelude::Queryable;
                                        use common::err::TransError;
                                        use common::log::error;
                                        let res = conn.query_first(#sql).hand_log(|msg| error!("数据库操作失败: {msg}"))?;
                                            Ok(res)
                                    })
                                        }
                                    }
                                }
                            }
                        }
                        Some(condition_param_vec) => {
                            let (param_name, param_type): (Vec<_>, Vec<_>) = condition_param_vec.iter().cloned().unzip();
                            match res_ident {
                                //指定了sql前缀,返回原始数据，需自己处理结果转换,res_type无效
                                None => {
                                    quote!(
                                    pub fn #fn_name_ident<T: mysql::prelude::FromRow>(conn: &mut mysql::PooledConn,#(#param_name:&#param_type),*)->common::err::GlobalResult<Option<T>>{
                                        use mysql::prelude::Queryable;
                                        use common::err::TransError;
                                        use common::log::error;
                                        use mysql::params;
                                        let res = conn.exec_first(#sql,params!{
                                            #(#param_name),*
                                        }).hand_log(|msg| error!("数据库操作失败: {msg}"))?;
                                    Ok(res)
                                    })
                                }
                                Some(vec) => {
                                    match inner.get_res_type() {
                                        Some(true) => {
                                            if vec.len() == field_infos.len() {
                                                quote!(
                                    pub fn #fn_name_ident(conn: &mut mysql::PooledConn,#(#param_name:&#param_type),*)->common::err::GlobalResult<Option<#struct_name>>{
                                        use mysql::prelude::Queryable;
                                        use common::err::TransError;
                                        use common::log::error;
                                        use mysql::params;
                                       let res = conn.exec_first(#sql,params!{
                                            #(#param_name),*}).hand_log(|msg| error!("数据库操作失败: {msg}"))?.map(|(#(#vec),*)|#struct_name{#(#vec),*});
                                            Ok(res)
                                    })
                                            } else {
                                                quote!(
                                    pub fn #fn_name_ident(conn: &mut mysql::PooledConn,#(#param_name:&#param_type),*)->common::err::GlobalResult<Option<#struct_name>>{
                                        use mysql::prelude::Queryable;
                                        use common::err::TransError;
                                        use common::log::error;
                                        use mysql::params;
                                       let res = conn.exec_first(#sql,params!{
                                            #(#param_name),*}).hand_log(|msg| error!("数据库操作失败: {msg}"))?.map(|(#(#vec),*)|#struct_name{#(#vec),*,..Default::default()});
                                            Ok(res)
                                    })
                                            }
                                        }
                                        _ => {
                                            quote!(
                                    pub fn #fn_name_ident<T: mysql::prelude::FromRow>(conn: &mut mysql::PooledConn,#(#param_name:&#param_type),*)->common::err::GlobalResult<Option<T>>{
                                        use mysql::prelude::Queryable;
                                        use common::err::TransError;
                                        use common::log::error;
                                        use mysql::params;
                                        let res = conn.exec_first(#sql,params!{
                                            #(#param_name),*
                                        }).hand_log(|msg| error!("数据库操作失败: {msg}"))?;
                                            Ok(res)
                                    })
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                SqlTypeExt::BATCH => {
                    match condition_param_name_type {
                        None => {
                            match res_ident {
                                //指定了sql前缀,返回原始数据，需自己处理结果转换,res_type无效
                                None => {
                                    quote!(
                                    pub fn #fn_name_ident<T: mysql::prelude::FromRow>(conn: &mut mysql::PooledConn,_limit_start:u32, _limit_end:u32)->common::err::GlobalResult<Vec<T>>{
                                        use mysql::prelude::Queryable;
                                        use common::err::TransError;
                                        use common::log::error;
                                        let res = conn.exec(#sql,params!{_limit_start,_limit_end}).hand_log(|msg| error!("数据库操作失败: {msg}"))?;
                                    Ok(res)
                                    })
                                }
                                Some(vec) => {
                                    match inner.get_res_type() {
                                        Some(true) => {
                                            if vec.len() == field_infos.len() {
                                                quote!(
                                    pub fn #fn_name_ident(conn: &mut mysql::PooledConn,_limit_start:u32, _limit_end:u32)->common::err::GlobalResult<Vec<#struct_name>>{
                                        use mysql::prelude::Queryable;
                                        use common::err::TransError;
                                        use common::log::error;
                                        let res = conn.exec(#sql,params!{_limit_start,_limit_end}).hand_log(|msg| error!("数据库操作失败: {msg}"))?.into_iter().map(|(#(#vec),*)|#struct_name{#(#vec),*}).collect();
                                            Ok(res)
                                    })
                                            } else {
                                                quote!(
                                    pub fn #fn_name_ident(conn: &mut mysql::PooledConn,_limit_start:u32, _limit_end:u32)->common::err::GlobalResult<Vec<#struct_name>>{
                                        use mysql::prelude::Queryable;
                                        use common::err::TransError;
                                        use common::log::error;
                                        let res = conn.exec(#sql,params!{_limit_start,_limit_end}).hand_log(|msg| error!("数据库操作失败: {msg}"))?.into_iter()
                                                .map(|(#(#vec),*)|#struct_name{#(#vec),*,..Default::default()}).collect();
                                            Ok(res)
                                    })
                                            }
                                        }
                                        _ => {
                                            quote!(
                                    pub fn #fn_name_ident<T: mysql::prelude::FromRow>(conn: &mut mysql::PooledConn,_limit_start:u32, _limit_end:u32)->common::err::GlobalResult<Vec<T>>{
                                        use mysql::prelude::Queryable;
                                        use common::err::TransError;
                                        use common::log::error;
                                        let res = conn.exec(#sql,params!{_limit_start,_limit_end}).hand_log(|msg| error!("数据库操作失败: {msg}"))?;
                                            Ok(res)
                                    })
                                        }
                                    }
                                }
                            }
                        }
                        Some(condition_param_vec) => {
                            let (param_name, param_type): (Vec<_>, Vec<_>) = condition_param_vec.iter().cloned().unzip();
                            match res_ident {
                                //指定了sql前缀,返回原始数据，需自己处理结果转换,res_type无效
                                None => {
                                    quote!(
                                    pub fn #fn_name_ident<T: mysql::prelude::FromRow>(conn: &mut mysql::PooledConn,#(#param_name:&#param_type),*,_limit_start:u32, _limit_end:u32)->common::err::GlobalResult<Vec<T>>{
                                        use mysql::prelude::Queryable;
                                        use common::err::TransError;
                                        use common::log::error;
                                        use mysql::params;
                                        let res = conn.exec(#sql,params!{
                                            #(#param_name),*,_limit_start,_limit_end
                                        }).hand_log(|msg| error!("数据库操作失败: {msg}"))?;
                                    Ok(res)
                                    })
                                }
                                Some(vec) => {
                                    match inner.get_res_type() {
                                        Some(true) => {
                                            if vec.len() == field_infos.len() {
                                                quote!(
                                    pub fn #fn_name_ident(conn: &mut mysql::PooledConn,#(#param_name:&#param_type),*,_limit_start:u32, _limit_end:u32)->common::err::GlobalResult<Vec<#struct_name>>{
                                        use mysql::prelude::Queryable;
                                        use common::err::TransError;
                                        use common::log::error;
                                        use mysql::params;
                                        let res = conn.exec(#sql,params!{
                                            #(#param_name),*,_limit_start,_limit_end}).hand_log(|msg| error!("数据库操作失败: {msg}"))?.into_iter().map(|(#(#vec),*)|#struct_name{#(#vec),*}).collect();
                                            Ok(res)
                                    })
                                            } else {
                                                quote!(
                                    pub fn #fn_name_ident(conn: &mut mysql::PooledConn,#(#param_name:&#param_type),*,_limit_start:u32, _limit_end:u32)->common::err::GlobalResult<Vec<#struct_name>>{
                                        use mysql::prelude::Queryable;
                                        use common::err::TransError;
                                        use common::log::error;
                                        use mysql::params;
                                        let res = conn.exec(#sql,params!{
                                            #(#param_name),*,_limit_start,_limit_end}).hand_log(|msg| error!("数据库操作失败: {msg}"))?.into_iter()
                                                .map(|(#(#vec),*)|#struct_name{#(#vec),*,..Default::default()}).collect();
                                            Ok(res)
                                    })
                                            }
                                        }
                                        _ => {
                                            quote!(
                                    pub fn #fn_name_ident<T: mysql::prelude::FromRow>(conn: &mut mysql::PooledConn,#(#param_name:&#param_type),*,_limit_start:u32, _limit_end:u32)->common::err::GlobalResult<Vec<T>>{
                                        use mysql::prelude::Queryable;
                                        use common::err::TransError;
                                        use common::log::error;
                                        use mysql::params;
                                        let res =  conn.exec(#sql,params!{
                                            #(#param_name),*,_limit_start,_limit_end
                                        }).hand_log(|msg| error!("数据库操作失败: {msg}"))?;
                                            Ok(res)
                                    })
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        _ => {
            match *sql_type_ext {
                SqlTypeExt::SINGLE => {
                    match condition_param_name_type {
                        None => {
                            match res_ident {
                                //指定了sql前缀,返回原始数据，需自己处理结果转换,res_type无效
                                None => {
                                    quote!(
                                    pub fn #fn_name_ident<T: mysql::prelude::FromRow>(conn: &mut mysql::PooledConn)->common::err::GlobalResult<Option<T>>{
                                        use mysql::prelude::Queryable;
                                        use common::err::TransError;
                                        use common::log::error;
                                        let res = conn.query_first(#sql).hand_log(|msg| error!("数据库操作失败: {msg}"))?;
                                        Ok(res)
                                    })
                                }
                                Some(vec) => {
                                    match inner.get_res_type() {
                                        Some(true) => {
                                            if vec.len() == field_infos.len() {
                                                quote!(
                                    pub fn #fn_name_ident(conn: &mut mysql::PooledConn)->common::err::GlobalResult<Option<#struct_name>>{
                                        use mysql::prelude::Queryable;
                                        use common::err::TransError;
                                        use common::log::error;
                                        let res = conn.query_first(#sql).hand_log(|msg| error!("数据库操作失败: {msg}"))?.map(|(#(#vec),*)|#struct_name{#(#vec),*});
                                            Ok(res)
                                    })
                                            } else {
                                                quote!(
                                    pub fn #fn_name_ident(conn: &mut mysql::PooledConn)->common::err::GlobalResult<Option<#struct_name>>{
                                        use mysql::prelude::Queryable;
                                        use common::err::TransError;
                                        use common::log::error;
                                        let res = conn.query_first(#sql).hand_log(|msg| error!("数据库操作失败: {msg}"))?.map(|(#(#vec),*)|#struct_name{#(#vec),*,..Default::default()});
                                            Ok(res)
                                    })
                                            }
                                        }
                                        _ => {
                                            quote!(
                                    pub fn #fn_name_ident<T: mysql::prelude::FromRow>(conn: &mut mysql::PooledConn)->common::err::GlobalResult<Option<T>>{
                                        use mysql::prelude::Queryable;
                                        use common::err::TransError;
                                        use common::log::error;
                                        let res = conn.query_first(#sql).hand_log(|msg| error!("数据库操作失败: {msg}"))?;
                                            Ok(res)
                                    })
                                        }
                                    }
                                }
                            }
                        }
                        Some(condition_param_vec) => {
                            let (param_name, param_type): (Vec<_>, Vec<_>) = condition_param_vec.iter().cloned().unzip();
                            match res_ident {
                                //指定了sql前缀,返回原始数据，需自己处理结果转换,res_type无效
                                None => {
                                    quote!(
                                    pub fn #fn_name_ident<T: mysql::prelude::FromRow>(conn: &mut mysql::PooledConn,#(#param_name:&#param_type),*)->common::err::GlobalResult<Option<T>>{
                                        use mysql::prelude::Queryable;
                                        use common::err::TransError;
                                        use common::log::error;
                                        use mysql::params;
                                        let res = conn.exec_first(#sql,params!{
                                            #(#param_name),*
                                        }).hand_log(|msg| error!("数据库操作失败: {msg}"))?;
                                    Ok(res)
                                    })
                                }
                                Some(vec) => {
                                    match inner.get_res_type() {
                                        Some(true) => {
                                            if vec.len() == field_infos.len() {
                                                quote!(
                                    pub fn #fn_name_ident(conn: &mut mysql::PooledConn,#(#param_name:&#param_type),*)->common::err::GlobalResult<Option<#struct_name>>{
                                        use mysql::prelude::Queryable;
                                        use common::err::TransError;
                                        use common::log::error;
                                        use mysql::params;
                                       let res = conn.exec_first(#sql,params!{
                                            #(#param_name),*}).hand_log(|msg| error!("数据库操作失败: {msg}"))?.map(|(#(#vec),*)|#struct_name{#(#vec),*});
                                            Ok(res)
                                    })
                                            } else {
                                                quote!(
                                    pub fn #fn_name_ident(conn: &mut mysql::PooledConn,#(#param_name:&#param_type),*)->common::err::GlobalResult<Option<#struct_name>>{
                                        use mysql::prelude::Queryable;
                                        use common::err::TransError;
                                        use common::log::error;
                                        use mysql::params;
                                       let res = conn.exec_first(#sql,params!{
                                            #(#param_name),*}).hand_log(|msg| error!("数据库操作失败: {msg}"))?.map(|(#(#vec),*)|#struct_name{#(#vec),*,..Default::default()});
                                            Ok(res)
                                    })
                                            }
                                        }
                                        _ => {
                                            quote!(
                                    pub fn #fn_name_ident<T: mysql::prelude::FromRow>(conn: &mut mysql::PooledConn,#(#param_name:&#param_type),*)->common::err::GlobalResult<Option<T>>{
                                        use mysql::prelude::Queryable;
                                        use common::err::TransError;
                                        use common::log::error;
                                        use mysql::params;
                                        let res = conn.exec_first(#sql,params!{
                                            #(#param_name),*
                                        }).hand_log(|msg| error!("数据库操作失败: {msg}"))?;
                                            Ok(res)
                                    })
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                SqlTypeExt::BATCH => {
                    match condition_param_name_type {
                        None => {
                            match res_ident {
                                //指定了sql前缀,返回原始数据，需自己处理结果转换,res_type无效
                                None => {
                                    quote!(
                                    pub fn #fn_name_ident<T: mysql::prelude::FromRow>(conn: &mut mysql::PooledConn)->common::err::GlobalResult<Vec<T>>{
                                        use mysql::prelude::Queryable;
                                        use common::err::TransError;
                                        use common::log::error;
                                        let res = conn.query(#sql).hand_log(|msg| error!("数据库操作失败: {msg}"))?;
                                    Ok(res)
                                    })
                                }
                                Some(vec) => {
                                    match inner.get_res_type() {
                                        Some(true) => {
                                            if vec.len() == field_infos.len() {
                                                quote!(
                                    pub fn #fn_name_ident(conn: &mut mysql::PooledConn)->common::err::GlobalResult<Vec<#struct_name>>{
                                        use mysql::prelude::Queryable;
                                        use common::err::TransError;
                                        use common::log::error;
                                        let res = conn.query(#sql).hand_log(|msg| error!("数据库操作失败: {msg}"))?.into_iter().map(|(#(#vec),*)|#struct_name{#(#vec),*}).collect();
                                            Ok(res)
                                    })
                                            } else {
                                                quote!(
                                    pub fn #fn_name_ident(conn: &mut mysql::PooledConn)->common::err::GlobalResult<Vec<#struct_name>>{
                                        use mysql::prelude::Queryable;
                                        use common::err::TransError;
                                        use common::log::error;
                                        let res = conn.query(#sql).hand_log(|msg| error!("数据库操作失败: {msg}"))?.into_iter()
                                                .map(|(#(#vec),*)|#struct_name{#(#vec),*,..Default::default()}).collect();
                                            Ok(res)
                                    })
                                            }
                                        }
                                        _ => {
                                            quote!(
                                    pub fn #fn_name_ident<T: mysql::prelude::FromRow>(conn: &mut mysql::PooledConn)->common::err::GlobalResult<Vec<T>>{
                                        use mysql::prelude::Queryable;
                                        use common::err::TransError;
                                        use common::log::error;
                                        let res = conn.query(#sql).hand_log(|msg| error!("数据库操作失败: {msg}"))?;
                                            Ok(res)
                                    })
                                        }
                                    }
                                }
                            }
                        }
                        Some(condition_param_vec) => {
                            let (param_name, param_type): (Vec<_>, Vec<_>) = condition_param_vec.iter().cloned().unzip();
                            match res_ident {
                                //指定了sql前缀,返回原始数据，需自己处理结果转换,res_type无效
                                None => {
                                    quote!(
                                    pub fn #fn_name_ident<T: mysql::prelude::FromRow>(conn: &mut mysql::PooledConn,#(#param_name:&#param_type),*)->common::err::GlobalResult<Vec<T>>{
                                        use mysql::prelude::Queryable;
                                        use common::err::TransError;
                                        use common::log::error;
                                        use mysql::params;
                                        let res = conn.exec(#sql,params!{
                                            #(#param_name),*
                                        }).hand_log(|msg| error!("数据库操作失败: {msg}"))?;
                                    Ok(res)
                                    })
                                }
                                Some(vec) => {
                                    match inner.get_res_type() {
                                        Some(true) => {
                                            if vec.len() == field_infos.len() {
                                                quote!(
                                    pub fn #fn_name_ident(conn: &mut mysql::PooledConn,#(#param_name:&#param_type),*)->common::err::GlobalResult<Vec<#struct_name>>{
                                        use mysql::prelude::Queryable;
                                        use common::err::TransError;
                                        use common::log::error;
                                        use mysql::params;
                                        let res = conn.exec(#sql,params!{
                                            #(#param_name),*}).hand_log(|msg| error!("数据库操作失败: {msg}"))?.into_iter().map(|(#(#vec),*)|#struct_name{#(#vec),*}).collect();
                                            Ok(res)
                                    })
                                            } else {
                                                quote!(
                                    pub fn #fn_name_ident(conn: &mut mysql::PooledConn,#(#param_name:&#param_type),*)->common::err::GlobalResult<Vec<#struct_name>>{
                                        use mysql::prelude::Queryable;
                                        use common::err::TransError;
                                        use common::log::error;
                                        use mysql::params;
                                        let res = conn.exec(#sql,params!{
                                            #(#param_name),*}).hand_log(|msg| error!("数据库操作失败: {msg}"))?.into_iter()
                                                .map(|(#(#vec),*)|#struct_name{#(#vec),*,..Default::default()}).collect();
                                            Ok(res)
                                    })
                                            }
                                        }
                                        _ => {
                                            quote!(
                                    pub fn #fn_name_ident<T: mysql::prelude::FromRow>(conn: &mut mysql::PooledConn,#(#param_name:&#param_type),*)->common::err::GlobalResult<Vec<T>>{
                                        use mysql::prelude::Queryable;
                                        use common::err::TransError;
                                        use common::log::error;
                                        use mysql::params;
                                        let res =  conn.exec(#sql,params!{
                                            #(#param_name),*
                                        }).hand_log(|msg| error!("数据库操作失败: {msg}"))?;
                                            Ok(res)
                                    })
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/*macro_rules! map_query_result {
    ($row:expr, $struct_name:ident { $($field:ident : $ty:ty),* }) => {
        $struct_name {
            $(
                $field: $row.take::<$ty, _>(stringify!($field)).unwrap(),
            )*
        }
    };
}*/

fn build_read_sql(inner: &Inner, table_name: &String, struct_table_field_map: &IndexMap<String, String>, field_infos: &IndexMap<String, Type>)
                  -> (String, Option<Vec<Ident>>, Option<Vec<(Ident, Type)>>) {
    let mut sql = String::new();
    let mut idents: Option<Vec<Ident>> = Option::None;
    let mut condition_param_name_type = Vec::new();
    if let Some(sql_str) = inner.get_pre_where_sql() {
        sql = sql_str.to_string();
        sql.push_str(" FROM ");
        sql.push_str(&*table_name);
    } else {
        sql.push_str("SELECT ");
        let (read_field_str, read_field_ident_vec) = build_read_fields_str_vec(struct_table_field_map, inner.get_fields());
        sql.push_str(&*read_field_str);
        sql.push_str(" FROM ");
        sql.push_str(&*table_name);
        idents = Some(read_field_ident_vec);
    }
    match inner.get_condition() {
        None => {
            match inner.get_order() {
                None => {
                    if let Some(true) = inner.get_page() {
                        sql.push_str(&*format!(" LIMIT :_limit_start, :_limit_end"));
                    }
                }
                Some(order_map) => {
                    sql.push_str(" ORDER BY ");
                    for (index, (key, or)) in order_map.iter().enumerate() {
                        let order_field = struct_table_field_map.get(key).expect(&*format!("{} invalid fields by order", key));
                        if index > 0 {
                            sql.push(',');
                        }
                        sql.push_str(&*order_field);
                        sql.push_str(&*format!(" {}", or.get_value()));
                    }
                    if let Some(true) = inner.get_page() {
                        sql.push_str(&*format!(" LIMIT :_limit_start, :_limit_end"));
                    }
                }
            }
        }
        Some(condition_map) => {
            sql.push_str(" WHERE ");
            for (condition_index, (condition_struct_field, condition)) in condition_map.iter().enumerate() {
                let condition_table_field = struct_table_field_map.get(condition_struct_field).expect(&*format!("{} invalid fields by condition", condition_struct_field));
                if condition_index > 0 {
                    sql.push_str(" AND ");
                }
                sql.push_str(condition_table_field);
                sql.push_str(condition.get_value());
                sql.push_str(&*format!(":{}", condition_struct_field));
                let condition_param_type = field_infos.get(condition_struct_field).expect(&*format!("fn = {},condition field {} is invalid", inner.get_fn_name(), condition_struct_field));
                condition_param_name_type.push((format_ident!("{}",condition_struct_field), condition_param_type.clone()));
            }
            match inner.get_order() {
                None => {
                    if let Some(true) = inner.get_page() {
                        sql.push_str(&*format!(" LIMIT :_limit_start, :_limit_end"));
                    }
                }
                Some(order_map) => {
                    sql.push_str(" ORDER BY ");
                    for (index, (key, or)) in order_map.iter().enumerate() {
                        let order_field = struct_table_field_map.get(key).expect(&*format!("{} invalid fields by order", key));
                        if index > 0 {
                            sql.push(',');
                        }
                        sql.push_str(&*order_field);
                        sql.push_str(&*format!(" {}", or.get_value()));
                    }
                    if let Some(true) = inner.get_page() {
                        sql.push_str(&*format!(" LIMIT :_limit_start, :_limit_end"));
                    }
                }
            }
        }
    }
    if condition_param_name_type.is_empty() {
        (sql, idents, None)
    } else {
        (sql, idents, Some(condition_param_name_type))
    }
}

fn build_update_constructor(inner: &Inner, field_infos: &IndexMap<String, Type>, _sql_type_ext: &SqlTypeExt, table_name: &String, struct_table_field_map: &IndexMap<String, String>) -> proc_macro2::TokenStream {
    let mut sql = String::from("UPDATE ");
    sql.push_str(table_name);
    sql.push_str(" SET ");
    let (update_str, struct_fields_vec, struct_field_ident_vec) = build_update_fields_vec(&struct_table_field_map, inner.get_fields());
    sql.push_str(&*update_str);
    let fn_name = inner.get_fn_name();
    let fn_name_ident = format_ident!("{fn_name}");
    match inner.get_condition() {
        None => {
            quote!(
                pub fn #fn_name_ident(&self,conn: &mut mysql::PooledConn){
                    use mysql::prelude::Queryable;
                    use common::err::TransError;
                    use common::log::error;
                    let _ = conn.exec_drop(#sql,params!{
                        #(#struct_fields_vec => &self.#struct_field_ident_vec()),*
                    }
                    ).hand_log(|msg| error!("数据库操作失败: {msg}"));
                })
        }
        Some(map) => {
            sql.push_str(" WHERE ");
            let mut params = Vec::new();
            let mut param_types = Vec::new();
            for (index, (field_name, condition)) in map.iter().enumerate() {
                if index > 0 {
                    sql.push_str(" AND ")
                }
                let table_field_name = struct_table_field_map.get(&*field_name).expect(&*format!("fn = {},condition field {} is invalid", &fn_name, &field_name));
                sql.push_str(&*table_field_name);
                sql.push_str(condition.get_value());
                sql.push_str(&*format!(":c_{}", &field_name));
                let ident = format_ident!("c_{}",&field_name);
                params.push(ident);
                param_types.push(field_infos.get(&*field_name).unwrap().clone());
            }
            quote!(
                pub fn #fn_name_ident(&self,conn: &mut mysql::PooledConn,#(#params:&#param_types),*){
                    use mysql::prelude::Queryable;
                    use common::err::TransError;
                    use mysql::params;
                    use common::log::error;
                    let _ = conn.exec_drop(#sql,params!{
                        #(#struct_fields_vec => &self.#struct_field_ident_vec()),*,
                        #(#params),*
                    }).hand_log(|msg| error!("数据库操作失败: {msg}"));
                }
            )
        }
    }
}

fn buidl_delete_constructor(inner: &Inner, field_infos: &IndexMap<String, Type>, table_name: &String, struct_table_field_map: &IndexMap<String, String>) -> proc_macro2::TokenStream {
    let mut sql = String::from("DELETE FROM ");
    sql.push_str(table_name);
    let fn_name = inner.get_fn_name();
    let fn_name_ident = format_ident!("{fn_name}");
    match inner.get_condition() {
        None => {
            quote!(
                pub fn #fn_name_ident(conn: &mut mysql::PooledConn){
                    use mysql::prelude::Queryable;
                    use common::err::TransError;
                    use common::log::error;
                    let _ = conn.query_drop(#sql).hand_log(|msg| error!("数据库操作失败: {msg}"));
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
                pub fn #fn_name_ident(conn: &mut mysql::PooledConn,#(#params:&#param_types),*){
                    use mysql::prelude::Queryable;
                    use common::err::TransError;
                    use mysql::params;
                    use common::log::error;
                    let _ = conn.exec_drop(#sql,params!{
                        #(#field_names => #params),*
                    }).hand_log(|msg| error!("数据库操作失败: {msg}"));
                }
            )
        }
    }
}

fn build_create_constructor(inner: &Inner, sql_type_ext: &SqlTypeExt, table_name: &String, struct_table_field_map: &IndexMap<String, String>) -> proc_macro2::TokenStream {
    let mut sql = String::from("INSERT INTO ");
    sql.push_str(table_name);
    sql.push_str(" (");
    let (table_field_name, struct_field_name, update_fields_create_str, struct_field_name_vec) = build_create_fields_str(&struct_table_field_map, inner.get_fields());
    sql.push_str(&*table_field_name);
    sql.push_str(") VALUES (");
    sql.push_str(&*struct_field_name);
    sql.push(')');
    if let Some(true) = inner.get_exist_update() {
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
                pub fn #fn_name_ident(&self, conn: &mut mysql::PooledConn) {
                    use mysql::prelude::Queryable;
                    use mysql::params;
                    use common::err::TransError;
                    use common::log::error;
                    let _ = conn.exec_drop(#sql,params!{
                        #(#struct_field_name_vec => &self.#field_value_ident()),*
                    }).hand_log(|msg| error!("数据库操作失败: {msg}"));
                }
            )
        }
        SqlTypeExt::BATCH => {
            quote!(
                pub fn #fn_name_ident(vec:Vec<Self>, conn: &mut mysql::PooledConn) {
                    use mysql::prelude::Queryable;
                    use mysql::params;
                    use common::err::TransError;
                    use common::log::error;
                    let _ = conn.exec_batch(#sql,vec.iter().map(|p|params!{
                       #(#struct_field_name_vec => &p.#field_value_ident()),*
                    })).hand_log(|msg| error!("数据库操作失败: {msg}"));
                }
            )
        }
    }
}

fn build_read_fields_str_vec(struct_table_field_map: &IndexMap<String, String>, fields: &Option<Vec<String>>) -> (String, Vec<Ident>) {
    let mut read_fields_str = String::new();
    let mut read_fields_ident = Vec::new();
    match fields {
        None => {
            for (index, (key, val)) in struct_table_field_map.iter().enumerate() {
                if index > 0 {
                    read_fields_str.push(',');
                }
                if key.eq(val) {
                    read_fields_str.push_str(&*key);
                } else {
                    read_fields_str.push_str(&*format!("{val} {key}"));
                }
                read_fields_ident.push(format_ident!("{key}"));
            }
            (read_fields_str, read_fields_ident)
        }
        Some(vec) => {
            for (index, struct_field_name) in vec.iter().enumerate() {
                if index > 0 {
                    read_fields_str.push(',');
                }
                let table_field_str = struct_table_field_map.get(struct_field_name).expect(&*format!("{} invalid fields", struct_field_name));
                if table_field_str.eq(struct_field_name) {
                    read_fields_str.push_str(&*struct_field_name);
                } else {
                    read_fields_str.push_str(&*format!("{table_field_str} {struct_field_name}"));
                }
                read_fields_ident.push(format_ident!("{struct_field_name}"));
            }
            (read_fields_str, read_fields_ident)
        }
    }
}

//返回update_str,struct_fields_vec,struct_field_ident_vec
fn build_update_fields_vec(struct_table_field_map: &IndexMap<String, String>, fields: &Option<Vec<String>>) -> (String, Vec<String>, Vec<Ident>) {
    let mut update_fields_str = String::new();
    let mut struct_fields_ident_vec = Vec::new();
    let mut struct_fields_vec = Vec::new();
    match fields {
        None => {
            for (index, (struct_field_str, table_field_str)) in struct_table_field_map.iter().enumerate() {
                if index > 0 {
                    update_fields_str.push(',');
                }
                update_fields_str.push_str(&*format!("{}=:{}", table_field_str, struct_field_str));
                struct_fields_ident_vec.push(format_ident!("get_{}",struct_field_str));
                struct_fields_vec.push(struct_field_str.clone());
            }
            (update_fields_str, struct_fields_vec, struct_fields_ident_vec)
        }
        Some(vec) => {
            for (index, struct_field_str) in vec.iter().enumerate() {
                if index > 0 {
                    update_fields_str.push(',');
                }
                let table_field_str = struct_table_field_map.get(struct_field_str).expect(&*format!("{} invalid fields", struct_field_str));
                update_fields_str.push_str(&*format!("{}=:{}", table_field_str, struct_field_str));
                struct_fields_ident_vec.push(format_ident!("get_{}",struct_field_str));
                struct_fields_vec.push(struct_field_str.clone());
            }
            (update_fields_str, struct_fields_vec, struct_fields_ident_vec)
        }
    }
}

/// struct_table_field_map -> struct_field_name:table_field_name
/// fields -> 指定field
/// 未指定field时，全部字段
/// table_fields_str,struct_fields_str,update_fields_create_str,struct_fields_vec
fn build_create_fields_str(struct_table_field_map: &IndexMap<String, String>, fields: &Option<Vec<String>>) -> (String, String, String, Vec<String>) {
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
fn build_struct_to_table_field_map(attr_state: &State, struct_info: &StructInfo) -> IndexMap<String, String> {
    let mut res_map: IndexMap<String, String> = IndexMap::new();
    match attr_state.get_alias_fields() {
        None => {
            if let Some(true) = attr_state.get_field_name_to_snake() {
                for (struct_field_name, _field_type) in struct_info.get_field_infos() {
                    res_map.insert(struct_field_name.clone(), to_snake_case(&*struct_field_name));
                }
            } else {
                for (struct_field_name, _field_type) in struct_info.get_field_infos() {
                    res_map.insert(struct_field_name.clone(), struct_field_name.clone());
                }
            }
        }
        Some(map) => {
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
    }
    res_map
}