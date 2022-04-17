use hashid_stack::prelude::*;

fn d() -> HashID<HashIdDefault, 0> {
  Default::default()
}

#[test]
fn should_fail_for_encoding_nothing() {
  assert_eq!("", d().encode([]), "should return None when encoding an empty array")
}

#[test]
#[should_panic]
fn should_fail_for_decoding_nothing() {
  let _: [u64; 0] = d().decode_fast("").unwrap();
}

#[test]
#[should_panic]
fn should_fail_for_decoding_invalid_id() {
  let _: [u64; 1] = d().decode_fast("f").unwrap();
}
