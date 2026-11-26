//! Assemble a deployable StreetChaves WebAssembly package through Foundation.
//!
//! The consumer supplies its generated WASM bundle and authorized local media.
//! Foundation validates and stages the files, then writes web-export.json.

use foundation_runtime::web_export::{WebExportBuilder, WebExportManifest};
use std::env;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::process;

fn main() {
    if let Err(error) = run() {
        eprintln!("street-chaves-web-export: {error}");
        process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.len() != 7 || args[0] == "--help" || args[0] == "-h" {
        eprintln!(
            "Usage:\n  cargo run --bin street-chaves-web-export -- \\\n            <manifest.json> <wasm-js> <wasm-module> <assets-root> \\\n            <index.html> <main.js> <output-dir>\n\n\
             Example:\n  cargo run --bin street-chaves-web-export -- \\\n            web/web-export.example.json pkg/street_chaves_rust.js \\\n            pkg/street_chaves_rust_bg.wasm web web/index.html web/main.js \\\n            dist/street-chaves-web"
        );
        if args.len() != 7 {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "expected seven arguments").into());
        }
        return Ok(());
    }

    let manifest_path = PathBuf::from(&args[0]);
    let javascript_source = PathBuf::from(&args[1]);
    let wasm_source = PathBuf::from(&args[2]);
    let assets_root = PathBuf::from(&args[3]);
    let html_source = PathBuf::from(&args[4]);
    let main_javascript_source = PathBuf::from(&args[5]);
    let output_dir = PathBuf::from(&args[6]);

    let manifest_json = fs::read_to_string(&manifest_path)?;
    let manifest = WebExportManifest::parse(&manifest_json)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error.to_string()))?;

    let application_id = manifest.application.id.clone();
    let application_name = manifest.application.name.clone();
    let javascript_destination = manifest.wasm.javascript_glue.clone();
    let wasm_destination = manifest.wasm.wasm_module.clone();

    // Register the WASM bundle using the destinations declared by the manifest.
    let mut builder = WebExportBuilder::new(application_id, application_name).wasm_bundle_files(
        javascript_source,
        wasm_source,
        javascript_destination,
        wasm_destination,
    );

    // Register every primary asset and each explicitly declared fallback.
    for asset in manifest.assets {
        let source = assets_root.join(&asset.path);
        let fallbacks = asset.fallbacks.clone();
        builder = builder.asset_file(asset, source);

        for fallback in fallbacks {
            let fallback_source = assets_root.join(&fallback);
            builder = builder.package_file(fallback_source, fallback);
        }
    }

    // The HTML shell and browser entry point are regular files in the package.
    builder = builder
        .package_file(html_source, "index.html")
        .package_file(main_javascript_source, "main.js");

    let written = builder
        .write_to(&output_dir)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error.to_string()))?;

    println!(
        "Created Foundation Web export for '{}' at {} ({} resources).",
        written.application.name,
        output_dir.display(),
        written.assets.len()
    );
    Ok(())
}
