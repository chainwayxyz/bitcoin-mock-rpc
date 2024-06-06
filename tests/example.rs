//! Example integration test.

mod common;

#[test]
fn example_test() {
	let _ = common::get_temp_public_keys();
	let _ = common::Config::new();
}
