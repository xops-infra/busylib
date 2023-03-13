use std::env;

use arc_swap::ArcSwap;
use once_cell::sync::Lazy;

pub type GlobalString = Lazy<ArcSwap<String>>;
pub type GlobalStaticStr = Lazy<ArcSwap<&'static str>>;

pub fn dev_mode() -> bool {
    env::args().nth(1) == Some("dev".into())
}

pub fn env_var_with_default(name: &str, default: &str) -> ArcSwap<String> {
    let val = match env::var(name) {
        Ok(s) => s,
        Err(_) => default.to_string(),
    };
    ArcSwap::from_pointee(val)
}
