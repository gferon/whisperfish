/* Copyright (C) 2018 Olivier Goffart <ogoffart@woboq.com>
Permission is hereby granted, free of charge, to any person obtaining a copy of this software and
associated documentation files (the "Software"), to deal in the Software without restriction,
including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense,
and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so,
subject to the following conditions:
The above copyright notice and this permission notice shall be included in all copies or substantial
portions of the Software.
THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT
NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES
OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
*/
use std::process::Command;
use std::env;
use std::path::Path;
use std::io::Write;

use failure::*;

fn qmake_query(var: &str) -> String {
    String::from_utf8(
        Command::new("qmake")
            .args(&["-query", var])
            .output()
            .expect("Failed to execute qmake. Make sure 'qmake' is in your path")
            .stdout,
    ).expect("UTF-8 conversion failed")
}

fn qml_to_qrc() -> Result<(), Error> {
    let out_dir = &env::var("OUT_DIR")?;
    let qml_path = &Path::new(&out_dir).join("qml.rs");

    let mut f = std::fs::File::create(qml_path)?;

    let mut read_dirs = std::collections::VecDeque::new();
    read_dirs.push_back(std::fs::read_dir("qml")?);

    write!(f, "qrc!{{qml_resources, \"/\" {{ ")?;

    while let Some(read_dir) = read_dirs.pop_front() {
        for entry in read_dir {
            let entry = entry?.path();
            println!("cargo:rerun-if-changed={:?}", entry);
            if entry.is_dir() {
                read_dirs.push_back(std::fs::read_dir(entry)?);
            } else if entry.is_file() {
                write!(f, "{:?},", entry)?;
            }
        }
    }

    write!(f, " }} }}")?;

    Ok(())
}

fn main() {
    let qt_include_path = qmake_query("QT_INSTALL_HEADERS");
    let qt_library_path = qmake_query("QT_INSTALL_LIBS");

    cpp_build::Config::new()
        .include(qt_include_path.trim())
        .build("src/main.rs");

    let macos_lib_search = if cfg!(target_os = "macos") {
        "=framework"
    } else {
        ""
    };
    let macos_lib_framework = if cfg!(target_os = "macos") { "" } else { "5" };

    println!(
        "cargo:rustc-link-search{}={}",
        macos_lib_search,
        qt_library_path.trim()
    );
    println!("cargo:rustc-link-lib{}=Qt{}Widgets", macos_lib_search, macos_lib_framework);
    println!("cargo:rustc-link-lib{}=Qt{}Gui", macos_lib_search, macos_lib_framework);
    println!("cargo:rustc-link-lib{}=Qt{}Core", macos_lib_search, macos_lib_framework);
    println!("cargo:rustc-link-lib{}=Qt{}Quick", macos_lib_search, macos_lib_framework);
    println!("cargo:rustc-link-lib{}=Qt{}Qml", macos_lib_search, macos_lib_framework);
    println!("cargo:rustc-link-lib{}=Qt{}QuickControls2", macos_lib_search, macos_lib_framework);

    qml_to_qrc().unwrap();
}
