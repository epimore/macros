mod util;

use proc_macro::TokenStream;
use std::fmt::{self, Debug, Display, format};
use proc_macro2::{Group, Literal, TokenTree};
use quote::__private::ext::RepToTokensExt;
use quote::ToTokens;
use syn::{parse_macro_input, Attribute, parse};
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
    let attr_state = parse_attr(attr);
    println!("------------  {attr_state:?}");
    println!("*************");

    println!("*************");
    let stream = item.clone();
    let item_input = parse_macro_input!(stream as DeriveInput);
    let struct_info = parse_item(item_input);
    println!("-------{:?}", struct_info);
    println!("*************");

    println!("over ++++++\n\n");

    // let mut constructors = quote!();
    for inner in attr_state.get_funs() {
        let (sql_type, ext) = inner.get_sql_type();
        match sql_type {
            SqlType::CREATE => {
                let mut sql = String::from("INSERT INTO ");
                sql.push_str(attr_state.get_table_name());
                sql.push_str(" (");

                match ext {
                    SqlTypeExt::SINGLE => {}
                    SqlTypeExt::BATCH => {}
                }
            }
            SqlType::READ => {}
            SqlType::UPDATE => {}
            SqlType::DELETE => {}
        }
    }


    item
}

fn build_fields_sql(attr_state: &State) {
    match attr_state.get_alias_fields() {
        None => {}
        Some(alias_fields) => {}
    }
}

fn build_field_name(){}