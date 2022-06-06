use std::env;
use std::path::PathBuf;
use std::process::{Command, exit};

fn main() {
    println!("cargo:rerun-if-changed=frontend/");

    let project_root: PathBuf = env::var_os("CARGO_MANIFEST_DIR").unwrap().into();
    let frontend_root = project_root.join("frontend");

    // choose npm build scripts to run depending on current profile
    let profile = env::var("PROFILE").unwrap();
    let build_args = match &profile[..] {
        "debug" => ["run", "build-dev"],
        "release" => ["run", "build"],
        _ => unreachable!(),
    };

    // build & bundle frontend project
    let status = Command::new("npm")
        .current_dir(&frontend_root)
        .arg("install")
		.status()
        .unwrap();

	if !status.success() {
		eprintln!("npm install Failure: {:?}", status);
		exit(-1)
	}

    let status = Command::new("npm")
        .current_dir(&frontend_root)
        .args(build_args)
		.status()
        .unwrap();

	if !status.success() {
		eprintln!("npm build Failure: {:?}", status);
		exit(-1)
	}
}
