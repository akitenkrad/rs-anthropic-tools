fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_log::test]
    fn test_hello_world() {
        main();
        tracing::info!("This is a test for the hello world function.");
        assert!(true)
    }
}
