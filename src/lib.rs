#![feature(rustc_private)]

use std::{collections::BTreeMap, path::Path};

use rustc_middle::ty::TyCtxt;
use rustc_session::config::Input;

extern crate rustc_abi;
extern crate rustc_ast;
extern crate rustc_const_eval;
extern crate rustc_data_structures;
extern crate rustc_driver;
extern crate rustc_driver_impl;
extern crate rustc_error_codes;
extern crate rustc_errors;
extern crate rustc_feature;
extern crate rustc_hash;
extern crate rustc_hir;
extern crate rustc_index;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_mir_dataflow;
extern crate rustc_session;
extern crate rustc_span;
extern crate rustc_type_ir;

pub mod compile_util;

pub fn analyze_path(path: &Path, conf: &AnalysisConfig) -> AnalysisResult {
    analyze_input(compile_util::path_to_input(path), conf)
}

#[derive(Default, Debug, Clone)]
pub struct AnalysisConfig {}

type OutputParam = ();

pub type AnalysisResult = BTreeMap<String, Vec<OutputParam>>;

pub fn analyze_input(input: Input, conf: &AnalysisConfig) -> AnalysisResult {
    let config = compile_util::make_config(input);
    compile_util::run_compiler(config, |tcx| analyze(tcx, conf)).unwrap()
}

pub fn analyze(tcx: TyCtxt<'_>, conf: &AnalysisConfig) -> AnalysisResult {
    // TODO: the analysis here
    todo!()
}
