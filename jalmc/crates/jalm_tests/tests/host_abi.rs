use std::fs;
use std::path::Path;

#[test]
fn host_abi_wit_contains_expected_interfaces() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let path = manifest_dir.join("../../../abi/host_abi_v0.wit");
    let contents = fs::read_to_string(&path).expect("host_abi_v0.wit readable");

    assert!(contents.contains("world jalm-host"));
    assert!(contents.contains("interface logging"));
    assert!(contents.contains("interface http"));
    assert!(contents.contains("interface net"));
    assert!(contents.contains("interface cancel"));
}
