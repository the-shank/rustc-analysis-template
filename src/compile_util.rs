use rustc_data_structures::sync::Lrc;
use std::{
    path::{Path, PathBuf},
    process::Command,
};

use etrace::ok_or;
use rustc_errors::FluentBundle;
use rustc_errors::{emitter::Emitter, registry::Registry, translation::Translate, Handler};
use rustc_feature::UnstableFeatures;
use rustc_hash::{FxHashMap, FxHashSet};
use rustc_interface::Config;
use rustc_middle::ty::TyCtxt;
use rustc_session::{
    config::{CheckCfg, CrateType, ErrorOutputType, Input, Options},
    EarlyErrorHandler,
};
use rustc_span::{edition::Edition, source_map::SourceMap};

pub fn path_to_input(path: &Path) -> Input {
    Input::File(path.to_path_buf())
}

fn toolchain_path(home: Option<String>, toolchain: Option<String>) -> Option<PathBuf> {
    home.and_then(|home| {
        toolchain.map(|toolchain| {
            let mut path = PathBuf::from(home);
            path.push("toolchains");
            path.push(toolchain);
            path
        })
    })
}

fn sys_root() -> String {
    std::env::var("SYSROOT")
        .ok()
        .map(PathBuf::from)
        .or_else(|| {
            let home = std::env::var("RUSTUP_HOME")
                .or_else(|_| std::env::var("MULTIRUST_HOME"))
                .ok();
            let toolchain = std::env::var("RUSTUP_TOOLCHAIN")
                .or_else(|_| std::env::var("MULTIRUST_TOOLCHAIN"))
                .ok();
            toolchain_path(home, toolchain)
        })
        .or_else(|| {
            Command::new("rustc")
                .arg("--print")
                .arg("sysroot")
                .output()
                .ok()
                .and_then(|out| String::from_utf8(out.stdout).ok())
                .map(|s| PathBuf::from(s.trim()))
        })
        .or_else(|| option_env!("SYSROOT").map(PathBuf::from))
        .or_else(|| {
            let home = option_env!("RUSTUP_HOME")
                .or(option_env!("MULTIRUST_HOME"))
                .map(ToString::to_string);
            let toolchain = option_env!("RUSTUP_TOOLCHAIN")
                .or(option_env!("MULTIRUST_TOOLCHAIN"))
                .map(ToString::to_string);
            toolchain_path(home, toolchain)
        })
        .map(|pb| pb.to_string_lossy().to_string())
        .unwrap()
}

pub fn make_config(input: Input) -> Config {
    let opts = find_deps();
    Config {
        opts: Options {
            maybe_sysroot: Some(PathBuf::from(sys_root())),
            search_paths: opts.search_paths,
            externs: opts.externs,
            unstable_features: UnstableFeatures::Allow,
            crate_types: vec![CrateType::Rlib],
            debug_assertions: false,
            edition: Edition::Edition2021,
            ..Options::default()
        },
        crate_cfg: FxHashSet::default(),
        crate_check_cfg: CheckCfg::default(),
        input,
        output_dir: None,
        output_file: None,
        ice_file: rustc_driver::ice_path().clone(),
        file_loader: None,
        locale_resources: rustc_driver_impl::DEFAULT_LOCALE_RESOURCES,
        lint_caps: FxHashMap::default(),
        parse_sess_created: Some(Box::new(|ps| {
            ps.span_diagnostic = Handler::with_emitter(Box::new(SilentEmitter));
        })),
        register_lints: None,
        override_queries: None,
        make_codegen_backend: None,
        registry: Registry::new(rustc_error_codes::DIAGNOSTICS),
    }
}

fn find_deps() -> Options {
    let mut args = vec!["a.rs".to_string()];

    let dir = std::env::var("DIR").unwrap_or_else(|_| ".".to_string());
    let dep = format!("{}/deps_crate/target/debug/deps", dir);
    if let Ok(dir) = std::fs::read_dir(&dep) {
        args.push("-L".to_string());
        args.push(format!("dependency={}", dep));

        for f in dir {
            let f = ok_or!(f, continue);
            let f = f.file_name().to_str().unwrap().to_string();
            if !f.ends_with(".rlib") {
                continue;
            }
            let i = f.find('-').unwrap();
            let name = f[3..i].to_string();
            let d = format!("{}={}/{}", name, dep, f);
            args.push("--extern".to_string());
            args.push(d);
        }
    }

    let mut handler = EarlyErrorHandler::new(ErrorOutputType::default());
    let matches = rustc_driver::handle_options(&handler, &args).unwrap();
    rustc_session::config::build_session_options(&mut handler, &matches)
}

pub fn run_compiler<R: Send, F: FnOnce(TyCtxt<'_>) -> R + Send>(config: Config, f: F) -> Option<R> {
    rustc_driver::catch_fatal_errors(|| {
        rustc_interface::run_compiler(config, |compiler| {
            compiler.enter(|queries| queries.global_ctxt().ok()?.enter(|tcx| Some(f(tcx))))
        })
    })
    .ok()?
}

struct SilentEmitter;

impl Translate for SilentEmitter {
    fn fluent_bundle(&self) -> Option<&Lrc<FluentBundle>> {
        None
    }

    fn fallback_fluent_bundle(&self) -> &FluentBundle {
        panic!()
    }
}

impl Emitter for SilentEmitter {
    fn emit_diagnostic(&mut self, _: &rustc_errors::Diagnostic) {}

    fn source_map(&self) -> Option<&Lrc<SourceMap>> {
        None
    }
}
