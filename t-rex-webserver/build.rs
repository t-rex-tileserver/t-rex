use std::env;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("fonts.rs");
    let mut f = File::create(&dest_path).unwrap();

    writeln!(
        f,
        "pub fn fonts() -> HashMap<&'static str, &'static [u8]> {{"
    )
    .unwrap();
    writeln!(
        f,
        "let mut fonts = HashMap::<&'static str, &'static [u8]>::new();"
    )
    .unwrap();
    for l1 in fs::read_dir("./src/static/fonts/").unwrap() {
        let l1fn = l1.unwrap().path();
        if l1fn.is_dir() {
            for pbf in fs::read_dir(l1fn).unwrap() {
                let pbfpath = pbf.unwrap().path();
                let inclpath = pbfpath.as_path().canonicalize().unwrap();
                let mut pbfcomp = pbfpath.components();
                pbfcomp.next();
                pbfcomp.next();
                pbfcomp.next();
                let keypath = pbfcomp.as_path();
                writeln!(
                    f,
                    "fonts.insert(\"{}\", include_bytes!(\"{}\"));",
                    keypath
                        .display()
                        .to_string()
                        .replace(std::path::MAIN_SEPARATOR, "/"),
                    inclpath
                        .display()
                        .to_string()
                        .replace(std::path::MAIN_SEPARATOR, "/")
                        .replace("//?/", "")
                )
                .unwrap();
            }
        }
    }
    writeln!(f, "fonts").unwrap();
    writeln!(f, "}}").unwrap();
}
