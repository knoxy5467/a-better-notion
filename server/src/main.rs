#![warn(rustdoc::private_doc_tests)]
#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
//! Server-Side API crate
fn main() {
    println!("hello world");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_main() {
        // Test the output of the main function
        // by capturing the printed output
        let mut output = Vec::new();
        std::io::stdout().write_all(&mut output).unwrap();
        main();
        assert_eq!(output, b"hello world\n");
    }
}
