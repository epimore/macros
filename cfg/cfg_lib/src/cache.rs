use std::fs::File;
use std::io::Read;
use std::sync::Arc;
use clap::{Arg, Command};
use once_cell::sync::Lazy;
static CONF: Lazy<Arc<String>> = Lazy::new(|| Arc::new(make()));

pub fn get_config() -> Arc<String> {
    CONF.clone()
}

fn make() -> String {
    // let path = get_conf_path();
    // let path = "/home/ubuntu20/code/rs/mv/github/epimore/gmv/stream/config.yml".to_string();
    // let path = "/home/ubuntu20/code/rs/mv/github/epimore/gmv/session/config.yml".to_string();
    let path = "/home/ubuntu20/code/rs/mv/github/epimore/macros/cfg/cfg_macro/cfg1.yaml".to_string();
    let mut file = File::open(path).expect("not found config file to open");
    let mut conf = String::new();
    file.read_to_string(&mut conf).expect("read file content to string failed");
    conf
}

fn get_conf_path() -> String {
    let matches = Command::new("MyApp")
        .version("1.0")
        .author("Kz. <kz986542@gmail.com>")
        .about("get the path about config file")
        .arg(Arg::new("config")
            .short('c')
            .long("config")
            .help("Path to configuration file")
            .default_value("./config.yml")
        )
        .get_matches();
    matches.try_get_one::<String>("config").expect("get config failed").expect("not found config").to_string()
}