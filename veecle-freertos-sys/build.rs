//! Build script for veecle-freertos-sys that generates bindings and optionally builds and links the FreeRTOS library.
use std::collections::HashMap;
#[cfg(feature = "link-freertos")]
use std::env::VarError;
#[cfg(feature = "link-freertos")]
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::{env, fs};

use anyhow::{Context, Result, bail};
use bindgen::Formatter;
use bindgen::callbacks::{ItemInfo, ItemKind, ParseCallbacks};
#[cfg(feature = "link-freertos")]
use walkdir::WalkDir;

// Allows setting the location and name of the library for cases where it is not compiled by this crate.
#[cfg(feature = "link-freertos")]
/// Name of the FreeRTOS library to link to (without `lib` prefix and file ending).
const LIB_FREERTOS_NAME_ENV_KEY: &str = "LIB_FREERTOS_NAME";
#[cfg(feature = "link-freertos")]
/// Directory containing the FreeRTOS library.
const LIB_FREERTOS_SEARCH_PATH_ENV_KEY: &str = "LIB_FREERTOS_SEARCH_PATH";

/// Path to the directory containing the `FreeRTOSConfig.h` file.
const FREERTOS_CONFIG_INCLUDE_PATH_ENV_KEY: &str = "FREERTOS_CONFIG_INCLUDE_PATH";
/// Path to the FreeRTOS kernel include directory.
const FREERTOS_KERNEL_INCLUDE_PATH_ENV_KEY: &str = "FREERTOS_KERNEL_INCLUDE_PATH";
/// Path to the FreeRTOS `portmacro` directory.
const FREERTOS_KERNEL_PORTMACRO_INCLUDE_PATH_ENV_KEY: &str =
    "FREERTOS_KERNEL_PORTMACRO_INCLUDE_PATH";
/// Path to the FreeRTOS heap implementation file.
#[cfg(feature = "link-freertos")]
const FREERTOS_HEAP_FILE_PATH_ENV_KEY: &str = "FREERTOS_HEAP_FILE_PATH";

/// One or more paths to additional include directories used when generating bindings and building the FreeRTOS library.
const FREERTOS_ADDITIONAL_INCLUDE_PATHS_ENV_KEY: &str = "FREERTOS_ADDITIONAL_INCLUDE_PATHS";
/// If set, all paths in `FREERTOS_ADDITIONAL_INCLUDE_PATHS` interpreted as relative to the set base path.
const FREERTOS_ADDITIONAL_INCLUDE_PATHS_BASE_ENV_KEY: &str =
    "FREERTOS_ADDITIONAL_INCLUDE_PATHS_BASE";

/// Path to a file whose contents will be prepended to the bindings `wrapper.h` file.
/// This is useful to add `defines` on which the includes of the wrapper rely on.
const BINDINGS_WRAPPER_PREPEND_EXTENSION_PATH_ENV_KEY: &str =
    "BINDINGS_WRAPPER_PREPEND_EXTENSION_PATH";

/// Communicates the location of the generated FreeRTOS bindings to dependent crates.
const FREERTOS_BINDINGS_LOCATION_ENV_KEY: &str = "FREERTOS_BINDINGS_LOCATION";

/// Contains all function renames applied by [`FunctionRenames`];
const FUNCTION_RENAMES: &[(&str, &str)] = &[
    ("pvPortMalloc", "__pvPortMalloc"),
    ("vTaskDelay", "__vTaskDelay"),
    ("vPortGetHeapStats", "__vPortGetHeapStats"),
];

/// The C source code contains complex comments with embedded code. Some of the embedded code looks like Markdown (e.g.
/// `array[index]`). Thus, this callback wraps all comments in code blocks to prevent `rustdoc` from interpreting the
/// comments as Markdown (and failing).
#[derive(Debug)]
struct WrapComments;

impl ParseCallbacks for WrapComments {
    fn process_comment(&self, comment: &str) -> Option<String> {
        Some(format!("\n```text\n\n{comment}\n```"))
    }
}

/// Allows renaming functions for seamless wrappers.
///
/// This is used to rename functions considered safe to be able to provide safe replacement wrappers.
#[derive(Debug)]
struct FunctionRenames(HashMap<&'static str, &'static str>);

impl ParseCallbacks for FunctionRenames {
    fn generated_name_override(&self, item_info: ItemInfo<'_>) -> Option<String> {
        match item_info.kind {
            ItemKind::Function => self.0.get(item_info.name).map(ToString::to_string),
            _ => None,
        }
    }
}

fn main() -> Result<()> {
    if env::var("DOCS_RS").as_deref() == Ok("1") {
        println!(
            "cargo::warning=docs.rs detected, using pre-generated bindings to avoid needing FreeRTOS code"
        );

        let in_path = PathBuf::from(env::var("CARGO_MANIFEST_PATH")?)
            .parent()
            .unwrap()
            .join("src/posix-sample-bindings.rs");
        let out_dir = PathBuf::from(env::var("OUT_DIR")?);
        let bindings_out_path = out_dir.join("bindings.rs");

        fs::create_dir_all(&out_dir)?;

        fs::copy(&in_path, &bindings_out_path)?;

        fs::write(
            out_dir.join("warning.md"),
            "\
                Pre-generated sample bindings for docs.rs documentation.\n\
                \n\
                <div class=warning>\n\
                \n\
                These bindings were generated with a specific FreeRTOS configuration and may not match your target platform.\n\
                Generate your own bindings by configuring the required environment variables for your project, then build them locally:\n\
                \n\
                ```sh\n\
                cargo doc -p veecle-freertos-sys --no-deps --open\n\
                ```\n\
                \n\
                </div>\n\
            ",
        )?;

        println!(
            "cargo::metadata={FREERTOS_BINDINGS_LOCATION_ENV_KEY}={}",
            bindings_out_path.to_str().unwrap()
        );

        return Ok(());
    }

    let freertos_kernel_include_path = read_env_var(FREERTOS_KERNEL_INCLUDE_PATH_ENV_KEY)?;
    println!("FreeRTOS kernel include path: {freertos_kernel_include_path}");
    let freertos_portmacro_path = {
        if let Ok(port_path) = read_env_var(FREERTOS_KERNEL_PORTMACRO_INCLUDE_PATH_ENV_KEY) {
            port_path
        } else {
            let mut freertos_kernel_path = PathBuf::from(&freertos_kernel_include_path);
            freertos_kernel_path.pop();
            find_freertos_port_dir(&freertos_kernel_path)?
                .to_str()
                .unwrap()
                .to_owned()
        }
    };
    println!("FreeRTOS portmacro path: {freertos_portmacro_path}");
    let freertos_config_path = read_env_var(FREERTOS_CONFIG_INCLUDE_PATH_ENV_KEY)?;
    println!("FreeRTOS config path: {freertos_config_path}");
    let freertos_additional_include_paths_base =
        read_env_var(FREERTOS_ADDITIONAL_INCLUDE_PATHS_BASE_ENV_KEY)
            .map_or(PathBuf::new(), |path| PathBuf::from(&path));
    println!(
        "FreeRTOS additional include paths base: {}",
        freertos_additional_include_paths_base.to_str().unwrap()
    );

    let freertos_additional_include_paths: Vec<PathBuf> =
        read_env_var(FREERTOS_ADDITIONAL_INCLUDE_PATHS_ENV_KEY)
            .map_or(Vec::new(), |paths| env::split_paths(&paths).collect())
            .iter()
            .map(|path| freertos_additional_include_paths_base.join(path))
            .collect();
    freertos_additional_include_paths
        .iter()
        .try_for_each(|path| check_dir_exists(path))?;
    println!("FreeRTOS additional include paths: {freertos_additional_include_paths:?}");

    #[cfg(feature = "link-freertos")]
    link_freertos(
        &freertos_kernel_include_path,
        &freertos_portmacro_path,
        &freertos_config_path,
        &freertos_additional_include_paths,
    )?;

    generate_bindings(
        &freertos_kernel_include_path,
        &freertos_portmacro_path,
        &freertos_config_path,
        &freertos_additional_include_paths,
    )?;

    Ok(())
}

/// Generates the bindings to the FreeRTOS kernel, including the macro shim.
fn generate_bindings(
    freertos_kernel_include_path: &str,
    freertos_portmacro_path: &str,
    freertos_config_path: &str,
    freertos_additional_include_paths: &[PathBuf],
) -> Result<()> {
    let host = read_env_var("HOST")?;
    let target = read_env_var("TARGET")?;
    let manifest_directory = PathBuf::from(read_env_var("CARGO_MANIFEST_DIR")?);

    if host != target {
        // TODO BINDGEN_EXTRA_CLANG_ARGS without target might be used to set include directories.
        let target_clang_args_env_key = format!("BINDGEN_EXTRA_CLANG_ARGS_{target}");
        let target_clang_args_env_no_dashes_key = target_clang_args_env_key.replace("-", "_");

        if let Err(error) = env::var(&target_clang_args_env_key)
            && let Err(error_no_dashes) = env::var(&target_clang_args_env_no_dashes_key)
        {
            println!(
                "cargo::warning=Crosscompiling without explicitly setting target include path for bindgen via \
                     `{target_clang_args_env_key}` (error: \"{error}\") or `{target_clang_args_env_no_dashes_key}` \
                     (error: \"{error_no_dashes}\")!"
            );
        }
    }

    let mut wrapper_h = fs::read_to_string("wrapper.h")?;

    println!(
        "cargo::rerun-if-changed={}",
        manifest_directory.join("wrapper.h").to_str().unwrap()
    );
    println!(
        "cargo::rerun-if-changed={}",
        manifest_directory.join("macro-shim.h").to_str().unwrap()
    );
    println!(
        "cargo::rerun-if-changed={}",
        manifest_directory.join("fallbacks.h").to_str().unwrap()
    );

    if let Ok(wrapper_h_prepend_extension_path) =
        read_env_var(BINDINGS_WRAPPER_PREPEND_EXTENSION_PATH_ENV_KEY)
    {
        let wrapper_h_prepend_extension_path = PathBuf::from(wrapper_h_prepend_extension_path);
        check_file_exists(&wrapper_h_prepend_extension_path)?;
        let mut wrapper_h_prepend_extension =
            fs::read_to_string(wrapper_h_prepend_extension_path).unwrap();
        wrapper_h_prepend_extension.push_str(&wrapper_h);
        wrapper_h = wrapper_h_prepend_extension;
    }

    let bindings = bindgen::Builder::default()
        .header_contents("wrapper.h", &wrapper_h)
        .use_core()
        .clang_arg(format!("-I{freertos_kernel_include_path}"))
        .clang_arg(format!("-I{freertos_portmacro_path}"))
        .clang_arg(format!("-I{freertos_config_path}"))
        .clang_args(
            freertos_additional_include_paths
                .iter()
                .map(|path| format!("-I{}", path.to_str().unwrap())),
        )
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .parse_callbacks(Box::new(FunctionRenames(HashMap::from_iter(
            FUNCTION_RENAMES.iter().cloned(),
        ))))
        .parse_callbacks(Box::new(WrapComments {}))
        // bindgen cannot parse macros with type casts (e.g. `#define STUFF ((unsigned long) 2000)`) without `clang_macro_fallback`.
        .clang_macro_fallback()
        // Places the artifacts macro expansion artifacts in the `OUT_DIR`.
        .clang_macro_fallback_build_dir(Path::new(&read_env_var("OUT_DIR")?))
        // Fitting macros to smaller types allows `usize::from(macro)`.
        // Using `from` enables compile-errors on configurations where this would truncate values.
        .fit_macro_constants(true)
        .formatter(Formatter::Prettyplease)
        .generate()
        .unwrap();

    let out_path = PathBuf::from(read_env_var("OUT_DIR")?).join("bindings.rs");

    bindings.write_to_file(&out_path)?;

    println!(
        "cargo::metadata={FREERTOS_BINDINGS_LOCATION_ENV_KEY}={}",
        out_path.to_str().unwrap()
    );

    Ok(())
}

/// Links (and builds, depending on env-vars) the FreeRTOS library.
#[cfg(feature = "link-freertos")]
fn link_freertos(
    freertos_kernel_include_path: &str,
    freertos_portmacro_path: &str,
    freertos_config_path: &str,
    freertos_additional_include_paths: &[PathBuf],
) -> Result<()> {
    println!("cargo:rerun-if-env-changed={LIB_FREERTOS_NAME_ENV_KEY}");
    println!("cargo:rerun-if-env-changed={LIB_FREERTOS_SEARCH_PATH_ENV_KEY}");

    match (
        env::var(LIB_FREERTOS_NAME_ENV_KEY),
        env::var(LIB_FREERTOS_SEARCH_PATH_ENV_KEY),
    ) {
        (Ok(lib_freertos_name), Ok(lib_freertos_search_path)) => {
            println!("cargo::rustc-link-search={lib_freertos_search_path}");
            println!("cargo::rustc-link-lib=static:-bundle={lib_freertos_name}");
        }
        (Err(VarError::NotPresent), Err(VarError::NotPresent)) => {
            build_freertos_lib(
                freertos_kernel_include_path,
                freertos_portmacro_path,
                freertos_config_path,
                freertos_additional_include_paths,
            )?;
        }
        (Err(error_name), Err(error_location)) => {
            bail!(
                "could not read environment variables \"{LIB_FREERTOS_NAME_ENV_KEY}\" ({error_name}) and \
                 \"{LIB_FREERTOS_SEARCH_PATH_ENV_KEY}\" ({error_location})"
            )
        }
        (Err(error_name), Ok(_)) => {
            bail!(
                "could not read environment variable \"{LIB_FREERTOS_NAME_ENV_KEY}\": {error_name}"
            )
        }
        (Ok(_), Err(error_location)) => {
            bail!(
                "could not read environment variable \"{LIB_FREERTOS_SEARCH_PATH_ENV_KEY}\": {error_location}"
            )
        }
    }
    Ok(())
}

/// Compiles the FreeRTOS library.
#[cfg(feature = "link-freertos")]
pub fn build_freertos_lib(
    freertos_kernel_include_path: &str,
    freertos_portmacro_path: &str,
    freertos_config_path: &str,
    freertos_additional_include_paths: &[PathBuf],
) -> Result<()> {
    let mut freertos_kernel_path = PathBuf::from(freertos_kernel_include_path);
    freertos_kernel_path.pop();
    let freertos_kernel_include_path = PathBuf::from(freertos_kernel_include_path);
    let freertos_portmacro_path = PathBuf::from(freertos_portmacro_path);

    let manifest_directory = PathBuf::from(read_env_var("CARGO_MANIFEST_DIR")?);

    let new_shim = manifest_directory.join("macro-shim.c");
    check_file_exists(&new_shim)?;

    let fallbacks_file = manifest_directory.join("fallbacks.c");
    check_file_exists(&fallbacks_file)?;

    // We're passing Some(1) because  we only want the `.c` files in the FreeRTOS kernel
    // directory.
    check_dir_exists(&freertos_kernel_path)?;
    let freertos_files = find_c_files(&freertos_kernel_path, Some(1));
    let port_files = find_c_files(&freertos_portmacro_path, None);

    let mut cc = cc::Build::new();

    // Header files:
    check_dir_exists(&freertos_kernel_include_path)?;
    check_dir_exists(&freertos_portmacro_path)?;
    check_dir_exists(Path::new(freertos_config_path))?;
    println!("Kernel include path: {freertos_kernel_include_path:?}");
    add_include_dir(&mut cc, freertos_kernel_include_path);
    println!("portmacro path: {freertos_portmacro_path:?}");
    add_include_dir(&mut cc, freertos_portmacro_path);
    println!("config: {freertos_config_path:?}");
    add_include_dir(&mut cc, freertos_config_path);
    freertos_additional_include_paths
        .iter()
        .for_each(|path| add_include_dir(&mut cc, path));

    // Source files:
    add_build_files(&mut cc, freertos_files);
    add_build_files(&mut cc, port_files);
    add_build_files(&mut cc, [new_shim]);
    if let Ok(freertos_heap_file_path) = read_env_var(FREERTOS_HEAP_FILE_PATH_ENV_KEY) {
        check_file_exists(Path::new(&freertos_heap_file_path))?;
        add_build_files(&mut cc, [freertos_heap_file_path]);
    } else {
        println!("cargo:warning=no FreeRTOS heap implementation set");
    }

    add_build_files(&mut cc, [fallbacks_file]);

    let out_path = read_env_var("OUT_DIR")?;
    cc.out_dir(&out_path);
    println!("cargo::rustc-link-search={out_path}");

    cc.try_compile("freertos")
        .context("Are the target headers available?")
}

/// Returns the path to the FreeRTOS port directory.
///
/// If the port directory is not set, it will be detected based on the current build target.
fn find_freertos_port_dir(freertos_dir: &Path) -> Result<PathBuf> {
    let port_folder = match (
        read_env_var("TARGET")?.as_str(),
        read_env_var("CARGO_CFG_TARGET_ARCH")?.as_str(),
        read_env_var("CARGO_CFG_TARGET_OS")?.as_str(),
    ) {
        // TODO: this might not be a perfect target mapping.
        (_, _, "linux" | "macos") => "ThirdParty/GCC/Posix",
        ("thumbv7m-none-eabi", _, _) => "GCC/ARM_CM3",
        // M4 cores without FPU use M3
        ("thumbv7em-none-eabi", _, _) => "GCC/ARM_CM3",
        ("thumbv7em-none-eabihf", _, _) => "GCC/ARM_CM4F",
        _ => {
            bail!("unknown target: '{}'", read_env_var("TARGET")?);
        }
    };

    Ok(freertos_dir.join("portable").join(port_folder))
}

/// Reads the environment variable if set.
///
/// Emits `rerun-if-changed` for the environment variable.
fn read_env_var(var_name: &str) -> Result<String> {
    println!("cargo:rerun-if-env-changed={var_name}");
    env::var(var_name).context(format!(
        "could not read environment variable \"{var_name}\""
    ))
}

/// Returns a list with all the `.c` files found recursively in a given directory.
///
/// If `max_depth` is `None`, all subdirectories are searched recursively.
#[cfg(feature = "link-freertos")]
fn find_c_files(dir: &Path, max_depth: Option<usize>) -> Vec<PathBuf> {
    WalkDir::new(dir)
        .max_depth(max_depth.unwrap_or(usize::MAX))
        .into_iter()
        .filter_map(|entry| {
            if let Ok(file) = entry {
                let path = file.path();
                if path.extension() == Some(OsStr::new("c")) {
                    return Some(path.to_path_buf());
                }
            }
            None
        })
        .collect()
}

/// Adds an include directory to `cc`, and all `.h` files to the watch list.
#[cfg(feature = "link-freertos")]
fn add_include_dir<P>(cc: &mut cc::Build, dir: P)
where
    P: AsRef<Path>,
{
    cc.include(&dir);
    WalkDir::new(&dir)
        .max_depth(1)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .for_each(|entry| {
            let f_name = entry.path();
            if f_name.extension() == Some(OsStr::new("h")) {
                println!("cargo:rerun-if-changed={}", f_name.to_str().unwrap());
            }
        });
}

/// Adds set of `.c` files to be built with `cc`, and includes them in cargo's watch list.
#[cfg(feature = "link-freertos")]
fn add_build_files<P>(cc: &mut cc::Build, files: P)
where
    P: IntoIterator,
    P::Item: AsRef<Path>,
{
    files.into_iter().for_each(|file| {
        cc.file(&file);
        println!("cargo:rerun-if-changed={}", file.as_ref().to_str().unwrap());
    });
}

/// Checks whether the directory exists or not.
fn check_dir_exists(path: &Path) -> Result<()> {
    if !path.is_dir() {
        bail!("Directory does not exist:{}", path.to_str().unwrap());
    }
    Ok(())
}

/// Checks whether the file exists or not.
fn check_file_exists(path: &Path) -> Result<()> {
    if !path.is_file() {
        bail!("File does not exist: {}", path.to_str().unwrap());
    }
    Ok(())
}
