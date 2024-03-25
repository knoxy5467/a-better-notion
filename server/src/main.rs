//! Server-Side API crate

#![warn(rustdoc::private_doc_tests)]
#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
fn main() {
    println!("hello world");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn does_it_test_main() {
        main();
        assert!(true);
    }
}
