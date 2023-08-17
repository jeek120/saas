use std::str::Bytes;

fn main() {
    Vec::new()
}


const OS: i32 = 0;
const LINE: &'static [u8] = if OS == 0 {b"\r\n"} else {b"\n"};

// const fn p() {
//     println!("123")
// }