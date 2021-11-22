// 添加链接器脚本

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rustc-link-arg=-Trustsbi-hifive-unmatched/src/u740.ld");
}
