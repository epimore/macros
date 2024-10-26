use serde::Deserialize;
use cfg_macro::conf;


#[derive(Debug, Deserialize)]
#[conf]
struct Cfg1 {
    name: String,
    version: String,
    features: Features,
}
#[derive(Debug, Deserialize)]
#[conf(path="/home/ubuntu20/code/rs/mv/github/epimore/macros/cfg/cfg_macro/cfg1.yaml")]
struct Cfg2 {
    name: String,
    version: String,
    features: Features,
}

#[derive(Debug, Deserialize)]
#[conf(path="/home/ubuntu20/code/rs/mv/github/epimore/macros/cfg/cfg_macro/cfg1.yaml",prefix="features")]
struct Features {
    logging: bool,
    metrics: bool,
}

#[test]
fn test_default_conf1(){
    let conf = Cfg1::init_by_conf();
    println!("{:?}",conf);
}
#[test]
fn test_target_conf2(){
    let conf = Cfg2::init_by_conf();
    println!("{:?}",conf);
}
#[test]
fn test_prefix_conf(){
    let conf = Features::init_by_conf();
    println!("{:?}",conf);
}