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

fn compile_java(class_name: &str, source: &str) -> Option<CompiledClass> {
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

  let source_path = dir.join(format!("{class_name}.java"));
  fs::write(&source_path, source).expect("failed to write java source");

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
  Some(CompiledClass {
    bytes: fs::read(class_path).expect("failed to read compiled class file"),
    dir,
  })
}

fn java_release() -> Option<String> {
  std::env::var("RUST_JVM_E2E_JAVA_RELEASE")
    .ok()
    .filter(|release| !release.trim().is_empty())
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

#[test]
fn e2e_parse_simple_main_class() {
  let Some(compiled) = compile_java(
    "SimpleMain",
    r#"
public class SimpleMain {
  public static void main(String[] args) {
    System.out.println("hello");
  }
}
"#,
  ) else {
    return;
  };

  let class_file = parse_complete(&compiled.bytes);

  assert_eq!(class_file.header.magic, 0xCAFEBABE);
  assert!(class_file.header.major >= 45);
  assert!(class_file.methods.methods_count >= 2);
}

#[test]
fn e2e_parse_constants_switches_and_annotations() {
  let Some(compiled) = compile_java(
    "EdgeCases",
    r#"
import java.lang.annotation.ElementType;
import java.lang.annotation.Retention;
import java.lang.annotation.RetentionPolicy;
import java.lang.annotation.Target;

@Retention(RetentionPolicy.RUNTIME)
@Target({ElementType.TYPE, ElementType.TYPE_USE, ElementType.TYPE_PARAMETER, ElementType.METHOD, ElementType.PARAMETER})
@interface Marker {
  int value();
  Class<?> type();
  String name() default "ok";
}

@Marker(value = 7, type = String.class)
public class EdgeCases<@Marker(value = 1, type = Object.class) T> {
  static final long L = 1234567890123L;
  static final double D = 1.25d;

  public static int denseSwitch(int x) {
    switch (x) {
      case 0:
        return 10;
      case 1:
        return 11;
      case 2:
        return 12;
      default:
        return -1;
    }
  }

  public static int sparseSwitch(int x) {
    switch (x) {
      case 1:
        return 10;
      case 100:
        return 20;
      case 1000:
        return 30;
      default:
        return -1;
    }
  }

  public @Marker(value = 2, type = String.class) String typeUse(@Marker(value = 3, type = String.class) String input) {
    return input;
  }
}
"#,
  ) else {
    return;
  };

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
}

#[test]
fn e2e_parse_record_class() {
  let Some(compiled) = compile_java(
    "Point",
    r#"
public record Point(int x, int y) {
  public int sum() {
    return x + y;
  }
}
"#,
  ) else {
    return;
  };

  let class_file = parse_complete(&compiled.bytes);

  assert!(class_file
    .attributes
    .attributes
    .iter()
    .any(|attribute| matches!(attribute, ClassFileAttribute::Record(_))));
}

#[test]
fn e2e_parse_inner_classes_exceptions_and_lambda() {
  let Some(compiled) = compile_java(
    "AdvancedFeatures",
    r#"
import java.io.IOException;
import java.util.function.Supplier;

public class AdvancedFeatures implements Runnable {
  enum Mode {
    ON,
    OFF
  }

  static class Box {
    int value;
  }

  public void run() {
    try {
      mayThrow();
    } catch (IOException error) {
      throw new RuntimeException(error);
    } finally {
      int ignored = 1;
    }
  }

  public static String lambda() {
    Supplier<String> supplier = () -> "ok";
    return supplier.get();
  }

  static void mayThrow() throws IOException {
  }
}
"#,
  ) else {
    return;
  };

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
}

#[test]
fn e2e_parse_arrays_numeric_ops_and_casts() {
  let Some(compiled) = compile_java(
    "RuntimeShapes",
    r#"
public class RuntimeShapes {
  public static long mix(int seed) {
    long value = seed;
    value = (value << 3) ^ 0xCAFE_BABEL;
    double d = value / 3.0d;
    return value + (long) d;
  }

  public static int arrays(Object input) {
    int[][] grid = new int[2][3];
    grid[1][2] = input instanceof String ? ((String) input).length() : 5;
    Object[] refs = new String[] {"a", "bb"};
    return grid[1][2] + refs.length;
  }
}
"#,
  ) else {
    return;
  };

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
}

#[test]
fn e2e_parse_interfaces_inheritance_and_generics() {
  let Some(compiled) = compile_java(
    "GenericChild",
    r#"
interface Named {
  default String label() {
    return "named";
  }
}

abstract class GenericBase<T extends Number> {
  abstract T value();
}

public class GenericChild extends GenericBase<Integer> implements Named {
  public Integer value() {
    return 42;
  }

  public String label() {
    return Named.super.label() + value();
  }
}
"#,
  ) else {
    return;
  };

  let class_file = parse_complete(&compiled.bytes);

  assert_eq!(class_file.interfaces.interfaces_count, 1);
  assert!(class_file
    .attributes
    .attributes
    .iter()
    .any(|attribute| matches!(attribute, ClassFileAttribute::Signature(_))));
  assert!(method_code_names(&class_file, "label").contains(&"invokespecial"));
  assert!(parse_all_class_files(&compiled.dir).len() >= 3);
}

#[test]
fn e2e_parse_sealed_hierarchy_when_supported_by_javac() {
  let Some(compiled) = compile_java(
    "SealedRoot",
    r#"
public sealed interface SealedRoot permits SealedLeaf {
}

final class SealedLeaf implements SealedRoot {
}
"#,
  ) else {
    return;
  };

  let class_file = parse_complete(&compiled.bytes);

  assert!(class_file
    .attributes
    .attributes
    .iter()
    .any(|attribute| matches!(attribute, ClassFileAttribute::PermittedSubclasses(_))));
}
