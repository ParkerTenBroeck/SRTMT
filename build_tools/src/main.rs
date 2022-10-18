use std::{path::PathBuf, process::Command};

fn main() {
    let mut args = std::env::args().skip(1).peekable(); // skip executable name

    let mut raw_binaries = Vec::new();
    let mut run_vm = false;
    let mut release = false;
    let mut unreconized = Vec::new();

    while let Some(string) = args.next() {
        match string.as_str().trim() {
            "run" => {
                run_vm = true;
            }
            "-B" => {
                if let Some(next) = args.peek() {
                    if next.starts_with('-') {
                        panic!("Expected list of packages to build not: {}", next);
                    }
                    for arg in next.split(',') {
                        let arg = arg.trim();
                        build_vm_binary(arg);
                        let path = create_raw_binary(arg);
                        raw_binaries.push(path);
                    }
                }
                args.next();
            }
            "--release" => {
                release = true;
            }
            "--debug" => {}
            _ => {
                unreconized.push(string);
            }
        }
    }

    if run_vm {
        let mut run_cmd = Command::new("cargo");
        run_cmd.current_dir(std::env::current_dir().unwrap());

        let mut str = String::new();
        for p in raw_binaries {
            if !str.is_empty() {
                str.push(',');
            }
            str.push_str(p.to_str().unwrap())
        }
        run_cmd.arg("run");
        if release {
            run_cmd.arg("--release");
        }

        run_cmd.arg("--package");
        run_cmd.arg("srtmt");

        run_cmd.args(unreconized);

        run_cmd.arg("--").arg("-B").arg(str);

        let _ = run_cmd.status().unwrap();
    }
}

pub fn build_vm_binary(name: &str) {
    let mut run_cmd = Command::new("cargo");
    run_cmd.current_dir(std::env::current_dir().unwrap());

    run_cmd
        .arg("+nightly")
        .arg("build")
        .arg("--release")
        .arg("--package")
        .arg(name)
        .arg("--target")
        .arg("mips.json")
        .arg("-Zbuild-std=core,compiler_builtins,alloc")
        .arg("-Zbuild-std-features=compiler-builtins-mem");

    let _ = run_cmd.status().unwrap();
}

pub fn create_raw_binary(name: &str) -> PathBuf {
    let llvm_tools = llvm_tools::LlvmTools::new().unwrap();
    let objcopy = llvm_tools.tool(&llvm_tools::exe("llvm-objcopy")).unwrap();

    let mut run_cmd = Command::new(objcopy);
    let mut path = std::env::current_dir().unwrap();
    path.push("target");
    path.push("mips");
    path.push("release");
    run_cmd.current_dir(path.clone());

    run_cmd
        .arg("-O")
        .arg("binary")
        .arg("-I")
        .arg("elf32-big")
        .arg(&format!("./{}", name))
        .arg(&format!("./{}.bin", name));

    let _ = run_cmd.status().unwrap();

    path.push(&format!("{}.bin", name));
    path
}
