use std::{
    ffi::OsStr,
    fs::{self, File},
    io::{Read, Write},
    path::Path,
};

use shaderc;

fn main() {
    println!("cargo:rerun-if-changed=shaders/src");
    compile_shaders();
    generate_shader_modules();
}

fn compile_shaders() {
    let out_folder: &Path = Path::new("./shaders/");
    let source_folder: &Path = Path::new("./shaders/src/");

    // delete all compiled shaders
    //if let Ok(directory) = out_folder.read_dir() {
    //    for entry in directory {
    //        let _ = entry.and_then(|entry| std::fs::remove_file(entry.path()));
    //    }
    //}

    let shader_sources = fs::read_dir(source_folder).unwrap();
    for shader_source in shader_sources {
        match shader_source {
            Err(_) => continue,
            Ok(file) => {
                let out_file_path = out_folder.join(file.file_name().to_str().unwrap());
                let file_path = file.path();

                let in_file = file_path.to_str().unwrap();

                let shader_kind = match file_path.extension().unwrap().to_str().unwrap() {
                    "vert" => shaderc::ShaderKind::Vertex,
                    "frag" => shaderc::ShaderKind::Fragment,
                    "glsl" => shaderc::ShaderKind::InferFromSource,
                    _ => continue,
                };

                let mut source = String::new();
                File::open(&file_path)
                    .unwrap()
                    .read_to_string(&mut source)
                    .unwrap();

                let compiler = shaderc::Compiler::new().unwrap();
                let mut options = shaderc::CompileOptions::new().unwrap();
                options.set_source_language(shaderc::SourceLanguage::GLSL);
                options.set_target_env(
                    shaderc::TargetEnv::Vulkan,
                    shaderc::EnvVersion::Vulkan1_3 as u32,
                );
                let binary_result = compiler
                    .compile_into_spirv(&source, shader_kind, in_file, "main", None)
                    .unwrap();

                let mut compiled_file = File::create(out_file_path).unwrap();
                compiled_file.write(binary_result.as_binary_u8()).unwrap();
            }
        }
    }
}

fn generate_shader_modules() {
    let module_file = "./src/graphics/shaders.rs";
    let out_folder = "./shaders";

    let mut module_file_handle = File::create(module_file).unwrap();
    write!(
        module_file_handle,
        "\
/* ------------------------*/
/*   AUTO GENERATED FILE   */
/* ------------------------*/
"
    )
    .unwrap();

    let shader_sources = fs::read_dir(out_folder).unwrap();
    for shader_source in shader_sources {
        match shader_source {
            Err(_) => panic!(),
            Ok(file) => {
                if file.file_type().unwrap().is_dir() {
                    continue;
                }

                let _file_name = file.file_name();
                let file_name = _file_name.to_str().unwrap();
                println!("file: {file_name:?}");

                let _path = file.path();
                let stem = _path.file_stem().and_then(OsStr::to_str).unwrap();
                let ext = _path.extension().and_then(OsStr::to_str).unwrap();
                let shader_type = match ext {
                    "vert" => "vertex",
                    "frag" => "fragment",
                    _ => "unknown",
                };

                write!(
                    module_file_handle,
                    "
#[allow(non_snake_case)]
pub mod {ext}_{stem}
{{
    vulkano_shaders::shader! {{
        ty: \"{shader_type}\",
        bytes: \"{out_folder}/{file_name}\",
    }}
}}
"
                )
                .unwrap()
            }
        }
    }

    write!(
        module_file_handle,
        "//{}",
        std::time::SystemTime::UNIX_EPOCH
            .elapsed()
            .unwrap()
            .subsec_nanos()
    )
    .unwrap();
}
