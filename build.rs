extern crate shlex;

use std::env;

fn main() {
    let cflags = env::var("DEP_RIOT_SYS_CFLAGS")
        .expect("DEP_RIOT_SYS_CFLAGS is not set, check whether riot-sys exports it.");
    let cflags = shlex::split(&cflags).expect("Odd shell escaping in CFLAGS");

    println!("cargo:rerun-if-env-changed=DEP_RIOT_SYS_CFLAGS");

    for flag in cflags.iter() {
        if flag.starts_with("-DMODULE_") {
            // Some modules like cmsis-dsp_StatisticsFunctions have funny characters
            println!(
                "cargo:rustc-cfg=riot_module_{}",
                flag[9..].to_lowercase().replace("-", "_")
            );
        }

        if flag == "-DDEVELHELP" {
            println!("cargo:rustc-cfg=riot_develhelp");
        }
    }

    let bindgen_output_file = env::var("DEP_RIOT_SYS_BINDGEN_OUTPUT_FILE")
        .expect("riot-sys did not provide BINDGEN_OUTPUT_FILE");

    let bindgen_output =
        std::fs::read_to_string(bindgen_output_file).expect("Failed to read BINDGEN_OUTPUT_FILE");

    const BOOLEAN_FLAGS: &[&str] = &[
        // This decides whether or not some fields are populated ... and unlike with other
        // structs, the zeroed default is not a good solution here. (It'd kind of work, but
        // it'd produce incorrect debug output).
        "CONFIG_AUTO_INIT_ENABLE_DEBUG",
    ];

    let parsed = syn::parse_file(&bindgen_output).expect("Failed to parse bindgen output");
    for item in &parsed.items {
        if let syn::Item::Const(const_) = item {
            // It's the easiest way to get something we can `contains`...
            let ident = const_.ident.to_string();
            if BOOLEAN_FLAGS.contains(&ident.as_str()) {
                if let syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Int(litint),
                    ..
                }) = &*const_.expr
                {
                    let value: usize = litint
                        .base10_parse()
                        .expect("Identifier is integer literal but not parsable");
                    if value != 0 {
                        println!("cargo:rustc-cfg=marker_{}", ident.to_lowercase());
                    }
                    continue;
                }
                panic!(
                    "Found {} but it's not the literal const it was expected to be",
                    ident
                );
            }
        }
    }
}
