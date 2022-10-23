extern crate cc;

fn main() {
    cc::Build::new().file("tests/lznt1.c").compile("libfoo.a");
}
