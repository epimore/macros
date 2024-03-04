use proc_macro::TokenStream;
use std::collections::HashMap;
use std::fmt;
use std::fmt::Debug;
use std::ops::Index;
use proc_macro2::TokenTree;
use quote::ToTokens;
use syn::{Data, DeriveInput, Type};
use constructor::{Get, Set};
use tools::{ASC, BATCH, Condition, CREATE, DELETE, DESC, Order, READ, SINGLE, SqlType, SqlTypeExt, UPDATE};

pub fn parse_item(derive_input: DeriveInput) -> StructInfo {
    let mut field_infos = HashMap::new();
    match derive_input.data {
        Data::Struct(ref s) => {
            match s.fields {
                syn::Fields::Named(ref fields) => {
                    for field in &fields.named {
                        let field_name = field.ident.as_ref().unwrap();
                        let field_type = &field.ty;
                        field_infos.insert(field_name.to_string(), field_type.clone());
                    }
                }
                syn::Fields::Unnamed(ref fields) => panic!("Unnamed struct is not supported"),
                syn::Fields::Unit => panic!("Unit struct is not supported"),
            }
        }
        Data::Enum(_) => { panic!("Enum is not supported") }
        Data::Union(_) => { panic!("Union is not supported") }
    }
    let struct_name = (&derive_input.ident).to_string();
    StructInfo {
        struct_name,
        field_infos,
    }
}

#[derive(Get)]
pub struct StructInfo {
    //结构体名称
    struct_name: String,
    //(字段名，字段类型)
    field_infos: HashMap<String, Type>,
}

impl Debug for StructInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "StructInfo {{ struct_name: {}, field_infos: [", &self.struct_name)?;
        for (i, item) in self.field_infos.iter().enumerate() {
            if i > 0 { write!(f, ", ")?; }
            write!(f, "{{{}:{:?}}}", item.0, item.1.to_token_stream().to_string())?;
        }
        write!(f, "])")
    }
}

pub fn parse_attr(attrs: TokenStream) -> State {
    let args = proc_macro2::TokenStream::from(attrs);
    let mut state: State = Default::default();
    let mut pre_ident = "".to_string();
    for attr in args {
        match attr {
            TokenTree::Group(funs) => {
                let mut vec = Vec::new();
                for fun in funs.stream() {
                    match fun {
                        TokenTree::Group(fun_inner) => {
                            let mut inner = Inner::default();
                            for arg in fun_inner.stream() {
                                match arg {
                                    TokenTree::Group(_) => {}
                                    TokenTree::Ident(func_ident) => {
                                        pre_ident = func_ident.to_string();
                                    }
                                    TokenTree::Punct(_) => {}
                                    TokenTree::Literal(func_literal) => {
                                        inner.parse_ident(&*pre_ident, &*func_literal.to_string());
                                    }
                                }
                            }
                            vec.push(inner);
                        }
                        TokenTree::Ident(_) => {}
                        TokenTree::Punct(_) => {}
                        TokenTree::Literal(_) => {}
                    }
                }
                state.set_funs(vec);
            }
            TokenTree::Ident(i) => {
                let ident = i.to_string();
                pre_ident = ident;
            }
            TokenTree::Punct(_) => {}
            TokenTree::Literal(l) => {
                state.parse_ident(&pre_ident, &l.to_string());
            }
        }
    }
    state
}

#[derive(Default, Set, Get)]
pub struct State {
    table_name: String,
    //结构体字段映射到表字段(struct_field_name,table_field_name)
    alias_fields: Option<HashMap<String, String>>,
    //alias_fields不受控制，默认None=false
    field_name_to_snake: Option<bool>,
    //定义多个方法
    funs: Vec<Inner>,
}

impl Debug for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "State {{ table_name: {}, alias_fields: ", &self.table_name)?;
        match &self.alias_fields {
            None => { write!(f, "None")?; }
            Some(vec) => {
                write!(f, "Some([")?;
                for (i, item) in vec.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{{{},{}}}", item.0, item.1)?;
                }
                write!(f, "])")?;
            }
        }
        write!(f, " ,field_name_to_snake: ")?;
        match &self.field_name_to_snake {
            None => { write!(f, "None")? }
            Some(val) => {
                write!(f, "Some({})", val)?;
            }
        }
        write!(f, " ,funs: [")?;
        for (i, item) in self.funs.iter().enumerate() {
            if i > 0 { write!(f, ", ")?; }
            write!(f, "{:?}", item)?;
        }
        write!(f, "]}}")
    }
}

impl State {
    fn parse_ident(&mut self, ident_str: &str, literal_str: &str) {
        let literal_str = &*literal_str.replace(" ", "").replace("\"", "");
        match &*ident_str.trim().to_ascii_uppercase() {
            "TABLE_NAME" => {
                self.set_table_name(literal_str.to_string());
            }
            "FIELD_NAME_TO_SNAKE" => {
                let b = literal_str.parse::<bool>().expect(&*format!("[{literal_str}] is invalid;field_name_to_snake arg value shoule be one of [true,false]"));
                self.set_field_name_to_snake(Some(b));
            }
            "ALIAS_FIELDS" => {
                let map = literal_split_by_colon(ident_str, literal_str);
                self.set_alias_fields(if map.is_empty() { panic!("alias_fields is invalid"); } else { Some(map) });
            }
            "FUNS" => {}
            other => { panic!("[{other}] is invalid;Arg name should be one of [table_name,alias_fields,field_name_to_snake,funs]") }
        }
    }
}

#[derive(Set, Get)]
pub struct Inner {
    //方法名
    fn_name: String,
    //SQL类型：
    sql_type: (SqlType, SqlTypeExt),
    //指定操作字段，NONE=ALL,可使用SQL函数以$包裹;eg:$count(*)
    fields: Option<Vec<String>>,
    //条件字段，NONE=不控制
    condition: Option<HashMap<String, Condition>>,
    //分页(page_num,page_size)
    page: Option<(u32, u32)>,
    order: Option<HashMap<String, Order>>,
    //未指定时，返回struct；可以是基本数据类型及元组;eg:(u32,String,bool)
    res_type: Option<String>,
}

impl Debug for Inner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Inner {{ fn_name: {}, sql_type: ({},{}), fields: ", &self.fn_name, &self.sql_type.0.get_value(), &self.sql_type.1.get_value())?;
        match &self.fields {
            None => { write!(f, "None")?; }
            Some(vec) => {
                write!(f, "Some([")?;
                for (i, item) in vec.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", item)?;
                }
                write!(f, "])")?;
            }
        }
        write!(f, " ,condition: ")?;
        match &self.condition {
            None => { write!(f, "None")? }
            Some(vec) => {
                write!(f, "Some([")?;
                for (i, item) in vec.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{{{},{}}}", item.0, item.1.get_value())?;
                }
                write!(f, "])")?;
            }
        }
        write!(f, ", page: ")?;
        match &self.page {
            None => { write!(f, "None")? }
            Some(val) => {
                write!(f, "Some(({},{}))", val.0, val.1)?;
            }
        }
        write!(f, ", order: ")?;
        match &self.order {
            None => { write!(f, "None")? }
            Some(vec) => {
                write!(f, "Some([")?;
                for (i, item) in vec.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{{{},{}}}", item.0, item.1.get_value())?;
                }
                write!(f, "])")?;
            }
        }
        write!(f, ", res_type: ")?;
        match &self.res_type {
            None => { write!(f, "None")? }
            Some(val) => {
                write!(f, "Some({})", val)?;
            }
        }
        write!(f, "}}")
    }
}

impl Inner {
    fn parse_ident(&mut self, ident_str: &str, literal_str: &str) {
        let literal_str = &*literal_str.replace(" ", "").replace("\"", "");
        match &*ident_str.trim().to_ascii_uppercase() {
            "FN_NAME" => { self.set_fn_name(literal_str.to_string()); }
            "SQL_TYPE" => {
                let map = literal_split_by_colon(ident_str, literal_str);
                if map.is_empty() {
                    panic!("Not has sql_type");
                }
                let (st, ste): (Vec<_>, Vec<_>) = map.iter().unzip();
                let sql_type = SqlType::match_type(&*st.get(0).unwrap());
                let sql_type_ext = SqlTypeExt::match_type(&*ste.get(0).unwrap());
                self.set_sql_type((sql_type, sql_type_ext));
            }
            "FIELDS" => { self.set_fields(Some(literal_split_by_comma(literal_str))) }
            "CONDITION" => {
                let map = literal_split_by_colon(ident_str, literal_str);
                if map.is_empty() { panic!("condition is invalid") }
                let condition = map.iter()
                    .map(|(table_field_name, condition_symbol)| (table_field_name.to_string(), Condition::match_type(&*condition_symbol)))
                    .collect::<HashMap<String, Condition>>();
                self.set_condition(Some(condition));
            }
            "PAGE" => {
                let map = literal_split_by_colon(ident_str, literal_str);
                if map.is_empty() {
                    panic!("page is invalid");
                }
                let (page_num, page_size): (Vec<_>, Vec<_>) = map.iter().unzip();
                let page_num = page_num.get(0).unwrap().parse::<u32>().expect("page_num is invalid");
                let page_size = page_size.get(0).unwrap().parse::<u32>().expect("page_size is invalid");
                self.set_page(Some((page_num, page_size)));
            }
            "ORDER" => {
                let map = literal_split_by_colon(ident_str, literal_str);
                if map.is_empty() { panic!("order is invalid") }
                let order = map.iter().map(|(table_field_name, order_arg)| (table_field_name.to_string(), Order::match_type(&*order_arg)))
                    .collect::<HashMap<String, Order>>();
                self.set_order(Some(order));
            }
            "RES_TYPE" => { self.set_res_type(Some(literal_str.to_string())) }
            _other => { panic!("function inside: invalid arg name") }
        }
    }
}

impl Default for Inner {
    fn default() -> Self {
        Self {
            fn_name: "".to_string(),
            sql_type: (SqlType::READ, SqlTypeExt::SINGLE),
            fields: None,
            condition: None,
            page: None,
            order: None,
            res_type: None,
        }
    }
}

fn literal_split_by_comma(literal: &str) -> Vec<String> {
    let vec = literal.split(",").map(|str| str.to_string()).collect::<Vec<String>>();
    vec
}

fn literal_split_by_colon(ident_str: &str, literal: &str) -> HashMap<String, String> {
    let mut map: HashMap<String, String> = HashMap::new();
    for single_str in literal.split(",").collect::<Vec<_>>() {
        match &*ident_str.to_ascii_uppercase() {
            "SQL_TYPE" => {
                match &*single_str.to_ascii_uppercase() {
                    UPDATE => { map.insert(tools::UPDATE.to_string(), SINGLE.to_string()); }
                    DELETE => { map.insert(tools::DELETE.to_string(), SINGLE.to_string()); }
                    cr_str => {
                        let tuple_cr_str = cr_str.split_once(":").expect(&*format!("[{cr_str}] is invalid;sql_type->[create,read] should have one subtype [single,batch]"));
                        match tuple_cr_str {
                            (READ, SINGLE) => { map.insert(tools::READ.to_string(), tools::SINGLE.to_string()); }
                            (READ, BATCH) => { map.insert(tools::READ.to_string(), tools::BATCH.to_string()); }
                            (CREATE, SINGLE) => { map.insert(tools::CREATE.to_string(), tools::SINGLE.to_string()); }
                            (CREATE, BATCH) => { map.insert(tools::CREATE.to_string(), tools::BATCH.to_string()); }
                            other => { panic!("[({}:{})] is invalid;should like [create:single,update,read:batch,delete]", other.0, other.1) }
                        }
                    }
                }
            }
            "ALIAS_FIELDS" => {
                let tuple_alias_fields_str = single_str.split_once(":").expect(&*format!("[{single_str}] is invalid;alias_fields should like [struct_field_name:table_field_name]"));
                map.insert(tuple_alias_fields_str.0.to_string(), tuple_alias_fields_str.1.to_string());
            }
            "CONDITION" => {
                let tuple_condition_str = single_str.split_once(":").expect(&*format!("[{single_str}] is invalid;condition should like [struct_field_name:condition],condition is one of [=,>,<,=<,=>]"));
                map.insert(tuple_condition_str.0.to_string(), tuple_condition_str.1.to_string());
            }
            "PAGE" => {
                let tuple_page_str = single_str.split_once(":").expect(&*format!("[{single_str}] is invalid;page should like [page_number:page_size];eg 1:100"));
                map.insert(tuple_page_str.0.to_string(), tuple_page_str.1.to_string());
            }
            "ORDER" => {
                let tuple_order_str = single_str.split_once(":").expect(&*format!("[{single_str}] is invalid;order should like [table_field_name1:ASC,table_field_name2:DESC]"));
                match tuple_order_str.1 {
                    DESC => { map.insert(tuple_order_str.0.to_string(), tools::DESC.to_string()); }
                    ASC => { map.insert(tuple_order_str.0.to_string(), tools::ASC.to_string()); }
                    other => { panic!("[{other}] is invalid;order suffix should be one of [asc,desc]") }
                }
            }
            other => { panic!("[{other}] is invalid;Arg name by split ':' should be one of [alias_fields,sql_type,alias_fields,condition,page,order]") }
        }
    }
    map
}

#[test]
fn test() {
    println!("res_type = {:?}", Inner::default());
}
