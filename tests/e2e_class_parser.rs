use rust_jvm::{
  structure::class::{
    ClassFile, ClassFileAttribute, ClassFileParser, Constant, MethodInfoAttribute,
  },
  util::hex::hex_utf8,
};

use std::{
  fs,
  path::{Path, PathBuf},
  process::{Command, Stdio},
  time::{SystemTime, UNIX_EPOCH},
};

struct CompiledClass {
  bytes: Vec<u8>,
  dir: PathBuf,
}

fn load_fixture_classes(class_name: &str) -> Option<Vec<CompiledClass>> {
  if let Some(dirs) = precompiled_classes_dirs() {
    return Some(
      dirs
        .into_iter()
        .map(|dir| {
          let class_path = class_file_path(&dir, class_name);
          CompiledClass {
            bytes: fs::read(class_path).expect("failed to read precompiled class fixture"),
            dir,
          }
        })
        .collect(),
    );
  }

  let millis = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .expect("system time should be after unix epoch")
    .as_millis();
  let dir = std::env::temp_dir().join(format!(
    "rust_jvm_e2e_{}_{}_{}",
    std::process::id(),
    class_name,
    millis
  ));
  fs::create_dir_all(&dir).expect("failed to create javac temp dir");

  let source_path = fixture_source_path(class_name);

  let mut command = Command::new("javac");
  if let Some(release) = java_release() {
    command.arg("--release").arg(release);
  }
  let output = command
    .arg("-d")
    .arg(&dir)
    .arg(&source_path)
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .output();

  let output = match output {
    Ok(output) => output,
    Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
      eprintln!("skipping class parser E2E test because javac is not installed");
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

  let class_path = class_file_path(&dir, class_name);
  Some(vec![CompiledClass {
    bytes: fs::read(class_path).expect("failed to read compiled class file"),
    dir,
  }])
}

fn precompiled_classes_dirs() -> Option<Vec<PathBuf>> {
  if let Some(root) = std::env::var("RUST_JVM_E2E_CLASSES_ROOT")
    .ok()
    .filter(|dir| !dir.trim().is_empty())
    .map(PathBuf::from)
  {
    let mut dirs: Vec<PathBuf> = fs::read_dir(root)
      .expect("failed to read precompiled class fixture root")
      .map(|entry| entry.expect("failed to read fixture root entry").path())
      .filter(|path| path.is_dir())
      .collect();
    dirs.sort();
    assert!(!dirs.is_empty(), "precompiled fixture root must contain class directories");
    return Some(dirs);
  }

  std::env::var("RUST_JVM_E2E_CLASSES_DIR")
    .ok()
    .filter(|dir| !dir.trim().is_empty())
    .map(|dir| vec![PathBuf::from(dir)])
}

fn java_release() -> Option<String> {
  std::env::var("RUST_JVM_E2E_JAVA_RELEASE")
    .ok()
    .filter(|release| !release.trim().is_empty())
}

fn fixture_source_path(class_name: &str) -> PathBuf {
  PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    .join("tests")
    .join("fixtures")
    .join("java")
    .join(format!("{class_name}.java"))
}

fn class_file_path(dir: &PathBuf, class_name: &str) -> PathBuf {
  class_name
    .split('.')
    .fold(dir.clone(), |path, part| path.join(part))
    .with_extension("class")
}

fn parse_all_class_files(dir: &Path) -> Vec<(PathBuf, ClassFile)> {
  let mut parsed = Vec::new();
  visit_class_files(dir, &mut |path| {
    let bytes = fs::read(path).expect("failed to read nested class file");
    parsed.push((path.to_path_buf(), parse_complete(&bytes)));
  });
  parsed
}

fn visit_class_files(dir: &Path, visitor: &mut impl FnMut(&Path)) {
  for entry in fs::read_dir(dir).expect("failed to read javac output dir") {
    let path = entry.expect("failed to read javac output entry").path();
    if path.is_dir() {
      visit_class_files(&path, visitor);
    } else if path.extension().is_some_and(|extension| extension == "class") {
      visitor(&path);
    }
  }
}

fn parse_complete(bytes: &[u8]) -> ClassFile {
  let mut parser = ClassFileParser::new();
  let (remaining, class_file) = parser.parse(bytes).expect("class file should parse");
  assert!(remaining.is_empty(), "parser should consume the full class file");
  class_file
}

fn method_code_names(class_file: &ClassFile, method_name: &str) -> Vec<&'static str> {
  class_file
    .methods
    .methods
    .iter()
    .find(|method| {
      matches!(
        class_file.constant_pool.get_class(method.name_index),
        Ok(Constant::Utf8 { bytes, .. }) if hex_utf8(bytes) == method_name
      )
    })
    .expect("method should exist")
    .attributes
    .attributes
    .iter()
    .find_map(|attribute| match attribute {
      MethodInfoAttribute::Code(code) => Some(code.code.iter().map(|byte| byte.name).collect()),
      _ => None,
    })
    .expect("method should have a Code attribute")
}

fn for_each_fixture_class(class_name: &str, mut check: impl FnMut(&CompiledClass)) {
  let Some(compiled_classes) = load_fixture_classes(class_name) else {
    return;
  };

  assert!(!compiled_classes.is_empty(), "at least one fixture class should be available");
  for compiled in &compiled_classes {
    check(compiled);
  }
}

#[test]
fn e2e_parse_simple_main_class() {
  for_each_fixture_class("SimpleMain", |compiled| {
    let class_file = parse_complete(&compiled.bytes);

    assert_eq!(class_file.header.magic, 0xCAFEBABE);
    assert!(class_file.header.major >= 45);
    assert!(class_file.methods.methods_count >= 2);
  });
}

#[test]
fn e2e_parse_constants_switches_and_annotations() {
  for_each_fixture_class("EdgeCases", |compiled| {
    let class_file = parse_complete(&compiled.bytes);

    assert!(class_file
      .constant_pool
      .constants
      .iter()
      .any(|constant| matches!(constant, Constant::Long { .. })));
    assert!(class_file
      .constant_pool
      .constants
      .iter()
      .any(|constant| matches!(constant, Constant::Double { .. })));
    assert!(method_code_names(&class_file, "denseSwitch").contains(&"tableswitch"));
    assert!(method_code_names(&class_file, "sparseSwitch").contains(&"lookupswitch"));
    assert!(class_file.attributes.attributes.iter().any(|attribute| {
      matches!(attribute, ClassFileAttribute::RuntimeVisibleAnnotations(_))
    }));
    assert!(class_file.attributes.attributes.iter().any(|attribute| {
      matches!(attribute, ClassFileAttribute::RuntimeVisibleTypeAnnotations(_))
    }));
  });
}

#[test]
fn e2e_parse_record_class() {
  for_each_fixture_class("Point", |compiled| {
    let class_file = parse_complete(&compiled.bytes);

    assert!(class_file
      .attributes
      .attributes
      .iter()
      .any(|attribute| matches!(attribute, ClassFileAttribute::Record(_))));
  });
}

#[test]
fn e2e_parse_inner_classes_exceptions_and_lambda() {
  for_each_fixture_class("AdvancedFeatures", |compiled| {
    let class_file = parse_complete(&compiled.bytes);

    assert!(class_file
      .attributes
      .attributes
      .iter()
      .any(|attribute| matches!(attribute, ClassFileAttribute::InnerClasses(_))));
    assert!(class_file
      .attributes
      .attributes
      .iter()
      .any(|attribute| matches!(attribute, ClassFileAttribute::BootstrapMethods(_))));
    assert!(method_code_names(&class_file, "lambda").contains(&"invokedynamic"));
    assert!(class_file.methods.methods.iter().any(|method| {
      method.attributes.attributes.iter().any(|attribute| {
        matches!(attribute, MethodInfoAttribute::Exceptions(_))
      })
    }));

    let parsed_classes = parse_all_class_files(&compiled.dir);
    assert!(parsed_classes.len() >= 3);
  });
}

#[test]
fn e2e_parse_arrays_numeric_ops_and_casts() {
  for_each_fixture_class("RuntimeShapes", |compiled| {
    let class_file = parse_complete(&compiled.bytes);
    let mix = method_code_names(&class_file, "mix");
    let arrays = method_code_names(&class_file, "arrays");

    assert!(mix.contains(&"lshl"));
    assert!(mix.contains(&"lxor"));
    assert!(mix.contains(&"ddiv"));
    assert!(mix.contains(&"d2l"));
    assert!(arrays.contains(&"multianewarray"));
    assert!(arrays.contains(&"instanceof"));
    assert!(arrays.contains(&"checkcast"));
    assert!(arrays.contains(&"anewarray"));
  });
}

#[test]
fn e2e_parse_interfaces_inheritance_and_generics() {
  for_each_fixture_class("GenericChild", |compiled| {
    let class_file = parse_complete(&compiled.bytes);

    assert_eq!(class_file.interfaces.interfaces_count, 1);
    assert!(class_file
      .attributes
      .attributes
      .iter()
      .any(|attribute| matches!(attribute, ClassFileAttribute::Signature(_))));
    assert!(method_code_names(&class_file, "label").contains(&"invokespecial"));
    assert!(parse_all_class_files(&compiled.dir).len() >= 3);
  });
}

#[test]
fn e2e_parse_sealed_hierarchy_when_supported_by_javac() {
  for_each_fixture_class("SealedRoot", |compiled| {
    let class_file = parse_complete(&compiled.bytes);

    assert!(class_file
      .attributes
      .attributes
      .iter()
      .any(|attribute| matches!(attribute, ClassFileAttribute::PermittedSubclasses(_))));
  });
}
