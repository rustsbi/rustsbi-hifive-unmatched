use serde_derive::Deserialize;
use serde_device_tree::{self, error::Result};
use alloc::collections::BTreeMap;

#[derive(Debug, Deserialize)]
struct Tree<'a> {
    #[serde(borrow)]
    aliases: BTreeMap<&'a str, &'a str>,
    chosen: Option<Chosen<'a>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct Chosen<'a> {
    stdout_path: Option<&'a str>,
}

pub unsafe fn parse_device_tree(dtb_pa: usize) -> Result<()> {
    let tree: Tree = serde_device_tree::from_raw(dtb_pa as *const u8)?;
    use crate::console::println;
    if let Some(chosen) = tree.chosen {
        if let Some(stdout_path) = chosen.stdout_path {
            println!("stdout path: {}", stdout_path);
        }
    }
    Ok(())
}
