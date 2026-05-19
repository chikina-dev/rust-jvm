use rust_jvm::{
    structure::class::{ClassFile, ClassFileParser},
    vm::vm::Vm,
};

use std::{
    fs,
    path::PathBuf,
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
        "rust_jvm_vm_phase9_{}_{}_{}",
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
            eprintln!("skipping VM phase9 test because javac is not installed");
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

    Some(dir.join(format!("{class_name}.class")))
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

#[test]
fn invokes_public_static_void_main_with_empty_string_args() {
    let Some(class_path) = compile_fixture("Phase9PublicMain") else {
        return;
    };
    let bytes = fs::read(class_path).expect("failed to read compiled class file");
    let class_file = parse_complete(&bytes);

    let mut vm = Vm::new();
    vm.load_class_file(&class_file)
        .expect("class should load into VM");

    assert_eq!(vm.invoke_public_static_main("Phase9PublicMain"), Ok(None));
    assert_eq!(vm.memory.stdout, vec!["0\n"]);
}
