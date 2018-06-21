extern crate safe_bindgen;
extern crate jni;
#[macro_use]
extern crate unwrap;

use jni::signature::{JavaType, Primitive};
use safe_bindgen::{Bindgen, LangJava};
use std::collections::HashMap;
use std::env;
use std::path::Path;

const BSD_MIT_LICENSE: &str =
    "// Copyright 2018 MaidSafe.net limited.\n\
     //\n\
     // This SAFE Network Software is licensed to you under the MIT license\n\
     // <LICENSE-MIT or http://opensource.org/licenses/MIT> or the Modified\n\
     // BSD license <LICENSE-BSD or https://opensource.org/licenses/BSD-3-Clause>,\n\
     // at your option. This file may not be copied, modified, or distributed\n\
     // except according to those terms. Please review the Licences for the\n\
     // specific language governing permissions and limitations relating to use\n\
     // of the SAFE Network Software.";

fn main() {
    if env::var("CARGO_FEATURE_BINDINGS").is_err() {
        return;
    }
    gen_bindings_java();
}

fn gen_bindings_java() {
    let target_dir = Path::new("bindings/java/system_uri");

    let mut type_map = HashMap::new();

    type_map.insert(
        "App", JavaType::Primitive(Primitive::Byte) // TODO: Please check this
    );


    let mut bindgen = unwrap!(Bindgen::new());
    let mut lang = LangJava::new(type_map);

    lang.set_namespace("net.maidsafe.system_uri");
    lang.set_model_namespace("net.maidsafe.system_uri");
    let mut outputs = HashMap::new();

    bindgen.source_file("src/lib.rs");
    lang.set_lib_name(unwrap!(env::var("CARGO_PKG_NAME")));
    unwrap!(bindgen.compile(&mut lang, &mut outputs, true));

    add_license_headers(&mut outputs);
    unwrap!(bindgen.write_outputs(target_dir, &outputs))
}

fn add_license_headers(outputs: &mut HashMap<String, String>) {
    for content in outputs.values_mut() {
        add_license_header(content);
    }
}

fn add_license_header(content: &mut String) {
    *content = format!("{}\n{}", BSD_MIT_LICENSE, content);
}
