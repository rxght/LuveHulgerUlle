use std::{
    ffi::OsStr,
    fs::{self, File},
    io::{Read, Write},
    path::Path,
};

fn main() {
    compile_shaders();
    generate_shader_modules();
}

fn compile_shaders() {
    println!("cargo:rerun-if-changed=*");
    let out_folder: &Path = Path::new("./shaders/");
    let source_folder: &Path = Path::new("./shaders/src/");

    let shader_sources = fs::read_dir(source_folder).unwrap();
    for shader_source in shader_sources {
        match shader_source {
            Err(_) => continue,
            Ok(file) => {
                let output_path = out_folder.join(file.file_name().to_str().unwrap());
                let source_path = file.path();

                let output_modification_time =
                    File::open(output_path.clone()).ok().and_then(|file| {
                        file.metadata()
                            .ok()
                            .and_then(|metadata| metadata.modified().ok())
                    });

                let source_modification_time = file
                    .metadata()
                    .ok()
                    .and_then(|metadata| metadata.modified().ok());

                // figure out if the output file already is up to date
                if let (Some(source_time), Some(output_time)) =
                    (source_modification_time, output_modification_time)
                {
                    println!("{source_time:?}, {output_time:?}");
                    if output_time >= source_time {
                        continue;
                    }
                }

                let in_file = source_path.to_str().unwrap();

                let shader_kind = match source_path.extension().unwrap().to_str().unwrap() {
                    "vert" => shaderc::ShaderKind::Vertex,
                    "frag" => shaderc::ShaderKind::Fragment,
                    "glsl" => shaderc::ShaderKind::InferFromSource,
                    _ => continue,
                };

                let mut source_content = String::new();
                File::open(&source_path)
                    .unwrap()
                    .read_to_string(&mut source_content)
                    .unwrap();

                let compiler = shaderc::Compiler::new().unwrap();
                let mut options = shaderc::CompileOptions::new().unwrap();
                options.set_source_language(shaderc::SourceLanguage::GLSL);
                options.set_target_env(
                    shaderc::TargetEnv::Vulkan,
                    shaderc::EnvVersion::Vulkan1_3 as u32,
                );
                #[cfg(debug_assertions)]
                options.set_generate_debug_info();
                let binary_result = compiler
                    .compile_into_spirv(
                        &source_content,
                        shader_kind,
                        in_file,
                        "main",
                        Some(&options),
                    )
                    .unwrap();

                let mut compiled_file = File::create(output_path).unwrap();
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
