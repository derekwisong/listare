use assert_cmd::Command;

#[test]
fn runs() {
    Command::cargo_bin("listare").unwrap().assert().success();
}
