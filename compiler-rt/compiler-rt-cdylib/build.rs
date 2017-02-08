extern crate gcc;

use std::env;
use std::path::Path;
use std::process::Command;

struct Sources {
    files: Vec<&'static str>,
}

impl Sources {
    fn new() -> Sources {
        Sources { files: Vec::new() }
    }

    fn extend(&mut self, sources: &[&'static str]) {
        self.files.extend(sources);
    }
}

fn main() {
    if !Path::new("compiler-rt/.git").exists() {
        let _ = Command::new("git").args(&["submodule", "update", "--init"])
                                   .status();
    }

    let target = env::var("TARGET").expect("TARGET was not set");
    let cfg = &mut gcc::Config::new();

    if target.contains("msvc") {
        cfg.define("__func__", Some("__FUNCTION__"));
    } else {
        cfg.flag("-fno-builtin");
        cfg.flag("-fomit-frame-pointer");
        cfg.flag("-ffreestanding");
    }

    let mut sources = Sources::new();

    sources.extend(&[
        "muldi3.c",
        "mulosi4.c",
        "mulodi4.c",
        "divsi3.c",
        "divdi3.c",
        "modsi3.c",
        "moddi3.c",
        "divmodsi4.c",
        "divmoddi4.c",
        "ashldi3.c",
        "ashrdi3.c",
        "lshrdi3.c",
        "udivdi3.c",
        "umoddi3.c",
        "udivmoddi4.c",
        "udivsi3.c",
        "umodsi3.c",
        "udivmodsi4.c",
        "adddf3.c",
        "addsf3.c",
        "powidf2.c",
        "powisf2.c",
        // 128 bit integers
        "lshrti3.c",
        "modti3.c",
        "muloti4.c",
        "multi3.c",
        "udivmodti4.c",
        "udivti3.c",
        "umodti3.c",
        "ashlti3.c",
        "ashrti3.c",
        "divti3.c",
        "floatsisf.c",
        "floatsidf.c",
        "floatdidf.c",
        "floatunsisf.c",
        "floatunsidf.c",
        "floatundidf.c",
        "fixsfsi.c",
        "fixsfdi.c",
        "fixdfsi.c",
        "fixdfdi.c",
        "fixunssfsi.c",
        "fixunssfdi.c",
        "fixunsdfsi.c",
        "fixunsdfdi.c",
    ]);

    let builtins_dir = Path::new("compiler-rt/lib/builtins");
    for src in sources.files.iter() {
        cfg.file(builtins_dir.join(src));
    }

    cfg.compile("libcompiler-rt.a");

    println!("cargo:rerun-if-changed=build.rs");

    for source in sources.files.iter() {
        println!("cargo:rerun-if-changed={}", builtins_dir.join(source).display());
    }
}
