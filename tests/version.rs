use super::*;

#[test]
fn version_flag_prints_version() {
  CommandBuilder::new("--version")
    .stdout_regex("ordwoj .*\n")
    .run();
}
