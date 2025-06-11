use std::{env, fs::File, io::Write, path::Path};

use zstd::stream::write::Encoder;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=dwarfs-0.12.4.exe");
    println!("cargo:rerun-if-changed=winfsp-x64-2.1.25156.dll");
    let out_dir = env::var("OUT_DIR").unwrap();

    let dest_path = Path::new(&out_dir).join("dwarfs.exe.zst");
    compress_to(include_bytes!("dwarfs-0.12.4.exe"), dest_path);

    let dest_path = Path::new(&out_dir).join("winfsp-x64.dll.zst");
    compress_to(include_bytes!("winfsp-x64-2.1.25156.dll"), dest_path);
}

fn compress_to(input: &[u8], output: impl AsRef<Path>) {
    let f = File::create(output).unwrap();
    let mut encoder = Encoder::new(f, 19).unwrap();
    encoder.write_all(input).unwrap();
    encoder.finish().unwrap();
}
