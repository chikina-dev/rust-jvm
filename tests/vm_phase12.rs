use rust_jvm::vm::{
    api::{ClassBytes, invoke_public_static_main_from_class_bytes, invoke_static_from_class_bytes},
    value::Value,
};

use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{SystemTime, UNIX_EPOCH},
};

fn fixture_source_path(class_name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("java")
        .join(format!("{class_name}.java"))
}

fn compile_fixture(class_name: &str) -> Option<PathBuf> {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "rust_jvm_vm_phase12_{}_{}_{}",
        std::process::id(),
        class_name,
        nanos
    ));
    fs::create_dir_all(&dir).expect("failed to create javac temp dir");

    let mut command = Command::new("javac");
    if let Ok(release) = std::env::var("RUST_JVM_E2E_JAVA_RELEASE") {
        if !release.trim().is_empty() {
            command.arg("--release").arg(release);
        }
    }

    let output = command
        .arg("-d")
        .arg(&dir)
        .arg(fixture_source_path(class_name))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output();

    let output = match output {
        Ok(output) => output,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            eprintln!("skipping VM phase12 test because javac is not installed");
            return None;
        }
        Err(error) => panic!("failed to run javac: {error}"),
    };

    if !output.status.success() {
        panic!(
            "javac failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    Some(dir)
}

fn class_bytes_from_dir(dir: &Path) -> Vec<ClassBytes> {
    let mut paths = fs::read_dir(dir)
        .expect("failed to read javac output dir")
        .map(|entry| entry.expect("failed to read javac output entry").path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "class")
        })
        .collect::<Vec<_>>();
    paths.sort();

    paths
        .into_iter()
        .map(|path| {
            let binary_name = path
                .file_stem()
                .and_then(|stem| stem.to_str())
                .expect("class file stem should be utf-8")
                .to_string();
            let bytes = fs::read(&path).expect("failed to read compiled class file");
            ClassBytes::new(binary_name, bytes)
        })
        .collect()
}

#[test]
fn invokes_static_method_from_class_byte_bundle() {
    let Some(dir) = compile_fixture("Phase7CrossClassMain") else {
        return;
    };
    let result = invoke_static_from_class_bytes(
        class_bytes_from_dir(&dir),
        "Phase7CrossClassMain",
        "main",
        "()I",
        vec![],
    )
    .expect("class byte bundle should execute");

    assert_eq!(result.return_value, Some(Value::Int(14)));
    assert!(result.stdout.is_empty());
    assert!(result.stderr.is_empty());
}

#[test]
fn invokes_public_static_main_from_class_byte_bundle() {
    let Some(dir) = compile_fixture("Phase9PublicMain") else {
        return;
    };
    let result =
        invoke_public_static_main_from_class_bytes(class_bytes_from_dir(&dir), "Phase9PublicMain")
            .expect("class byte bundle should execute public main");

    assert_eq!(result.return_value, None);
    assert_eq!(result.stdout, vec!["0\n"]);
    assert!(result.stderr.is_empty());
}
