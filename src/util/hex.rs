
pub fn hex_viewer(bytes: &[u8]) -> String {
  bytes
    .iter()
    .enumerate()
    .map(|(i, b)| {
      let sep = if i > 0 && i % 4 == 3 { "," } else { "" };
      let line = if i > 0 && i % 16 == 15 { "\n" } else { "" };
      format!("{:02X}{} {}", b, sep, line)
    })
    .collect::<Vec<_>>()
    .join("")
}

pub fn hex_utf8(bytes: &[u8]) -> String {
  match String::from_utf8(bytes.to_vec()) {
    Ok(s) => s,
    Err(_) => {
      bytes.iter()
        .map(|b| format!("\\x{:02X}", b))
        .collect()
    }
  }
}