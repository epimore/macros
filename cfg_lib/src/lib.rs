pub mod util;
pub mod cache;
extern crate cfg_macro;
pub use cfg_macro::conf as conf;

pub trait Conf{
    fn conf()->Self where Self: Sized;
}