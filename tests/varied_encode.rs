use hashid_stack::prelude::*;

#[test]
fn empty() {
  test_salt(b"")
}
#[test]
fn spaces() {
  test_salt(b"   ")
}
#[test]
fn ordinary() {
  test_salt(b"this is my salt")
}
#[test]
fn long() {
  test_salt(b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890`~!@#$%^&*()-_=+\\|'\";:/?.>,<{[}]");
}
#[test]
fn garbage() {
   test_salt(b"`~!@#$%^&*()-_=+\\|'\";:/?.>,<{[}]")
}



fn test_salt<const SALT: usize>(salt: &[u8; SALT]) {
  macro_rules! hihi {
    ($t:ty, $hi:ident, $data:expr) => {
      let data = $data;
      let enc = $hi.encode(data);
      println!("{}({}).encode: {:?}, {}", stringify!($t), SALT, enc, enc.len());
      let dec = $hi.decode_fast(enc);
      println!("{}({}).decode: {:?} ({})", stringify!($t), SALT, dec, data.len());
      assert_eq!(data, dec.expect(stringify!($t)), concat!(stringify!($t), " didn't work?"));  
    }
  }
  macro_rules! hiii {
    ($($t:ty),*) => {$(
      let hi = <$t>::with_salt(salt);
      hihi!($t, hi, [1]);
      hihi!($t, hi, [1, 2]);
      hihi!($t, hi, [1, 2, 3]);
      hihi!($t, hi, [1, 2, 3, 4]);
      hihi!($t, hi, [1000, 2000, 3000, 4000]);
      hihi!($t, hi, [1024, 2048, 4096, 16384, 65535]);
      // 1..1<<25
      hihi!($t, hi, [1,2,4,8,16,32,64,128,256,512,1024,2048,4096,8192,16384,32768,65536,131072,262144,524288,1048576,2097152,4194304,8388608,16777216,33554432]);
      //1<<53 repeat 9
      hihi!($t, hi, [4503599627370496,4503599627370496,4503599627370496,4503599627370496,4503599627370496,4503599627370496,4503599627370496,4503599627370496,4503599627370496]);
      //i64::MAX repeat 7
      hihi!($t, hi, [9223372036854775807,9223372036854775807,9223372036854775807,9223372036854775807,9223372036854775807,9223372036854775807,9223372036854775807]);

      // random tests
      hihi!($t, hi, [12446646867894078354,4908001284546428738,9309420877296939278,15195939167604779550,5163634649444262478]);
      hihi!($t, hi, [5181379796382792916,6351314966946035469,7129577023501132124,4782098177175539108,9101063531486816439]);
      hihi!($t, hi, [5281073705171475060,10572695610645112184,2550522317539421180,4905622022567923489,590388389556150252,2228473606001573272]);

    )*}
  }
  hiii!(HashIdDefault, HashIdQr, HashIdB32, HashIdB64);
}