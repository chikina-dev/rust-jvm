use rust_jvm::vm::{
    loader::{ClassLoader, FileSystemClassSource, InMemoryClassSource},
    value::Value,
    vm::Vm,
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
        "rust_jvm_vm_phase10_{}_{}_{}",
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
            eprintln!("skipping VM phase10 test because javac is not installed");
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

fn in_memory_source_from_class_dir(dir: &Path) -> InMemoryClassSource {
    let mut paths = fs::read_dir(dir)
        .expect("failed to read javac output dir")
        .map(|entry| entry.expect("failed to read javac output entry").path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "class")
        })
        .collect::<Vec<_>>();
    paths.sort();

    let mut source = InMemoryClassSource::new();
    for path in paths {
        let binary_name = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .expect("class file stem should be utf-8");
        let bytes = fs::read(&path).expect("failed to read compiled class file");
        source.insert_class_bytes(binary_name, bytes);
    }
    source
}

#[test]
fn loads_classes_from_in_memory_class_source() {
    let Some(dir) = compile_fixture("Phase7CrossClassMain") else {
        return;
    };
    let source = in_memory_source_from_class_dir(&dir);
    let mut loader = ClassLoader::new(source);
    let mut vm = Vm::new();

    let class_ids = loader
        .load_classes(["Phase7CrossClassMain", "Phase7Helper"], &mut vm.memory)
        .expect("classes should load from in-memory source");
    let duplicate_id = vm
        .load_class_from_source(&mut loader, "Phase7CrossClassMain")
        .expect("duplicate class load should return existing class id");

    assert_eq!(class_ids[0], duplicate_id);
    assert_eq!(
        vm.invoke_static("Phase7CrossClassMain", "main", "()I", vec![]),
        Ok(Some(Value::Int(14)))
    );
}

#[test]
fn loads_classes_from_file_system_class_source() {
    let Some(dir) = compile_fixture("Phase7CrossClassMain") else {
        return;
    };
    let source = FileSystemClassSource::new(&dir);
    let mut loader = ClassLoader::new(source);
    let mut vm = Vm::new();

    vm.load_class_from_source(&mut loader, "Phase7CrossClassMain")
        .expect("main class should load from file system source");
    vm.load_class_from_source(&mut loader, "Phase7Helper")
        .expect("helper class should load from file system source");

    assert_eq!(
        vm.invoke_static("Phase7CrossClassMain", "main", "()I", vec![]),
        Ok(Some(Value::Int(14)))
    );
}

#[test]
fn reports_missing_class_from_in_memory_class_source() {
    let mut loader = ClassLoader::default();
    let mut vm = Vm::new();

    assert!(
        vm.load_class_from_source(&mut loader, "MissingClass")
            .is_err_and(|error| error
                == rust_jvm::vm::error::VmError::ClassNotFound("MissingClass".to_string()))
    );
}
