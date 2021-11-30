use std::{env, path::{Path, PathBuf}, process::{self, Command}};

use clap::{clap_app, crate_authors, crate_description, crate_version};

#[derive(Debug)]
struct XtaskEnv {
    compile_mode: CompileMode,
}

#[derive(Debug)]
enum CompileMode {
    Debug,
    Release
}

const DEFAULT_TARGET: &'static str = "riscv64imac-unknown-none-elf";

fn main() {
    let matches = clap_app!(xtask =>
        (version: crate_version!())
        (author: crate_authors!())
        (about: crate_description!())
        (@subcommand make =>
            (about: "Build project")
            (@arg release: --release "Build artifacts in release mode, with optimizations")
        )
        (@subcommand gdb =>
            (about: "Run GDB debugger")
        )
    ).get_matches();
    let mut xtask_env = XtaskEnv {
        compile_mode: CompileMode::Debug,
    };
    println!("xtask: mode: {:?}", xtask_env.compile_mode);
    if let Some(matches) = matches.subcommand_matches("make") {
        if matches.is_present("release") {
            xtask_env.compile_mode = CompileMode::Release;
        }
        xtask_build_sbi(&xtask_env);
        xtask_binary_sbi(&xtask_env);
    } else if let Some(_matches) = matches.subcommand_matches("gdb") {
        xtask_build_sbi(&xtask_env);
        xtask_binary_sbi(&xtask_env);
        xtask_unmatched_gdb(&xtask_env);
    } else {
        println!("Use `cargo make` to build, `cargo xtask --help` for help")
    }
}

fn xtask_build_sbi(xtask_env: &XtaskEnv) {
    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let mut command = Command::new(cargo);
    command.current_dir(project_root().join("rustsbi-hifive-unmatched"));
    command.arg("build");
    match xtask_env.compile_mode {
        CompileMode::Debug => {},
        CompileMode::Release => { command.arg("--release"); },
    }
    command.args(&["--package", "rustsbi-hifive-unmatched"]);
    command.args(&["--target", DEFAULT_TARGET]);
    let status = command
        .status().unwrap();
    if !status.success() {
        println!("cargo build failed");
        process::exit(1);
    }
}

fn xtask_binary_sbi(xtask_env: &XtaskEnv) {
    let objcopy = "rust-objcopy";
    let status = Command::new(objcopy)
        .current_dir(dist_dir(xtask_env))
        .arg("rustsbi-hifive-unmatched")
        .arg("--binary-architecture=riscv64")
        .arg("--strip-all")
        .args(&["-O", "binary", "rustsbi-hifive-unmatched.bin"])
        .status().unwrap();

    if !status.success() {
        println!("objcopy binary failed");
        process::exit(1);
    }
}

fn xtask_unmatched_gdb(xtask_env: &XtaskEnv) {
    let mut command = Command::new("riscv-none-embed-gdb");
    command.current_dir(dist_dir(xtask_env));
    command.args(&["--eval-command", "file rustsbi-hifive-unmatched"]);
    command.args(&["--eval-command", "target extended-remote localhost:3333"]);
    command.arg("--quiet");
        
    ctrlc::set_handler(move || {
        // when ctrl-c, don't exit gdb
    }).expect("disable Ctrl-C exit");

    let status = command.status().expect("run program");

    if !status.success() {
        println!("gdb failed with status {}", status);
        process::exit(status.code().unwrap_or(1));
    }
}

fn dist_dir(xtask_env: &XtaskEnv) -> PathBuf {
    let mut path_buf = project_root().join("target").join(DEFAULT_TARGET);
    path_buf = match xtask_env.compile_mode {
        CompileMode::Debug => path_buf.join("debug"),
        CompileMode::Release => path_buf.join("release"),
    };
    path_buf
}

fn project_root() -> PathBuf {
    Path::new(&env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(1)
        .unwrap()
        .to_path_buf()
}
