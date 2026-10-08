//! Embeds everything under `assets/` into the binary, so the app is one file.

use std::{env, fs, path::Path};

fn main() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets");
    println!("cargo:rerun-if-changed=assets");
    let mut files = Vec::new();
    collect(&root, &root, &mut files);
    files.sort();
    let mut out = String::from("pub static FILES: &[(&str, &[u8])] = &[\n");
    for rel in &files {
        let abs = root.join(rel);
        out.push_str(&format!(
            "    ({rel:?}, include_bytes!({:?})),\n",
            abs.to_string_lossy()
        ));
    }
    out.push_str("];\n");
    let dest = Path::new(&env::var("OUT_DIR").unwrap()).join("assets.rs");
    fs::write(dest, out).unwrap();
}

fn collect(root: &Path, dir: &Path, files: &mut Vec<String>) {
    for entry in fs::read_dir(dir).unwrap().flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect(root, &path, files);
        } else {
            let rel = path
                .strip_prefix(root)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/");
            files.push(rel);
        }
    }
}
