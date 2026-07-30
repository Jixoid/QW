use std::process::Command;
use std::fs;
use std::path::PathBuf;

fn get_qw_binary() -> PathBuf {
	let mut path = std::env::current_exe().unwrap();
	path.pop();
	if path.ends_with("deps") {
		path.pop();
	}
	path.push("QW"); // The binary is QW
	path
}

#[test]
fn test_ui_pass() {
	let pass_dir = PathBuf::from("tests/ui/pass");
	if !pass_dir.exists() {
		return;
	}

	let entries = fs::read_dir(pass_dir).unwrap();
	for entry in entries {
		let entry = entry.unwrap();
		let path = entry.path();
		if path.is_dir() {
			println!("Testing PASS: {:?}", path);
			let output = Command::new(get_qw_binary())
				.arg("check")
				.arg("--path")
				.arg(&path)
				.output()
				.expect("Failed to execute qw command");

			if !output.status.success() {
				let stdout = String::from_utf8_lossy(&output.stdout);
				let stderr = String::from_utf8_lossy(&output.stderr);
				panic!("Test {:?} was expected to pass but failed.\nstdout:\n{}\nstderr:\n{}", path, stdout, stderr);
			}
		}
	}
}

#[test]
fn test_ui_fail() {
	let fail_dir = PathBuf::from("tests/ui/fail");
	if !fail_dir.exists() {
		return;
	}

	let entries = fs::read_dir(fail_dir).unwrap();
	for entry in entries {
		let entry = entry.unwrap();
		let path = entry.path();
		if path.is_dir() {
			println!("Testing FAIL: {:?}", path);
			let output = Command::new(get_qw_binary())
				.arg("check")
				.arg("--path")
				.arg(&path)
				.output()
				.expect("Failed to execute qw command");

			let stdout = String::from_utf8_lossy(&output.stdout).to_lowercase();
			let stderr = String::from_utf8_lossy(&output.stderr).to_lowercase();
			let output_str = format!("{}\n{}", stdout, stderr);
            
			// A simple heuristic: if it contains an error or fatal message.
			if output.status.success() && !output_str.contains("error") && !output_str.contains("fatal") {
				panic!("Test {:?} was expected to fail but succeeded without error/fatal messages.\nstdout:\n{}\nstderr:\n{}", path, stdout, stderr);
			}
		}
	}
}
