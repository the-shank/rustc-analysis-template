use std::{
    fs,
    path::{Path, PathBuf},
    time::Instant,
};

use clap::Parser;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    dump_analysis_result: Option<PathBuf>,

    #[arg(short, long)]
    use_analysis_result: Option<PathBuf>,

    #[arg(short, long)]
    verbose: bool,

    #[arg(short, long)]
    transform: bool,

    #[arg(long)]
    time: bool,

    #[arg(short, long)]
    log_file: Option<PathBuf>,

    #[arg(short, long)]
    output: Option<PathBuf>,

    input: PathBuf,
}

fn main() {
    let mut args = Args::parse();
    args.time = true;
    args.output = Some(args.input.join("..").join("rw_outdir"));
    let _ = fs::create_dir_all(args.output.as_ref().unwrap());

    dbg!(&args);

    let path = if let Some(output) = &mut args.output {
        output.push(args.input.file_name().unwrap());
        if output.exists() {
            assert!(output.is_dir());
            clear_dir(output);
        } else {
            fs::create_dir(&output).unwrap();
        }
        copy_dir(&args.input, output, true);
        output
    } else {
        &mut args.input
    };

    if path.is_dir() {
        path.push("src/lib.rs");
    }
    assert!(path.is_file());

    let _t = Timer::new(args.time);
}

struct Timer {
    show: bool,
    start: Instant,
}

impl Timer {
    fn new(show: bool) -> Self {
        Self {
            show,
            start: Instant::now(),
        }
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        if self.show {
            println!("{:.3}s", self.start.elapsed().as_secs_f64());
        }
    }
}

fn clear_dir(path: &Path) {
    for entry in fs::read_dir(path).unwrap() {
        let entry_path = entry.unwrap().path();
        if entry_path.is_dir() {
            let name = entry_path.file_name().unwrap();
            if name != "target" {
                fs::remove_dir_all(entry_path).unwrap();
            }
        } else {
            fs::remove_file(entry_path).unwrap();
        }
    }
}

fn copy_dir(src: &Path, dst: &Path, root: bool) {
    for entry in fs::read_dir(src).unwrap() {
        let src_path = entry.unwrap().path();
        let name = src_path.file_name().unwrap();
        let dst_path = dst.join(name);
        if src_path.is_file() {
            fs::copy(src_path, dst_path).unwrap();
        } else if src_path.is_dir() && (!root || name != "target") {
            fs::create_dir(&dst_path).unwrap();
            copy_dir(&src_path, &dst_path, false);
        }
    }
}
