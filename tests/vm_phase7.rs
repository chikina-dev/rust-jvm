use rust_jvm::{
    structure::class::{ClassFile, ClassFileParser},
    vm::{value::Value, vm::Vm},
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
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_millis();
    let dir = std::env::temp_dir().join(format!(
        "rust_jvm_vm_phase7_{}_{}_{}",
        std::process::id(),
        class_name,
        millis
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
            eprintln!("skipping VM phase7 test because javac is not installed");
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

fn parse_complete(bytes: &[u8]) -> ClassFile {
    let mut parser = ClassFileParser::new();
    let (remaining, class_file) = parser.parse(bytes).expect("class file should parse");
    assert!(
        remaining.is_empty(),
        "parser should consume the full class file"
    );
    class_file
}

fn parse_class_files(dir: &Path) -> Vec<ClassFile> {
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
            let bytes = fs::read(path).expect("failed to read compiled class file");
            parse_complete(&bytes)
        })
        .collect()
}

#[test]
fn invokes_static_method_from_another_loaded_class() {
    let Some(dir) = compile_fixture("Phase7CrossClassMain") else {
        return;
    };
    let class_files = parse_class_files(&dir);

    let mut vm = Vm::new();
    vm.load_class_files(&class_files)
        .expect("classes should load into VM");

    assert_eq!(
        vm.invoke_static("Phase7CrossClassMain", "main", "()I", vec![]),
        Ok(Some(Value::Int(14)))
    );
}
