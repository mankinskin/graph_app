pub mod read;

#[cfg(test)]
pub mod tests;

fn main() {
    #[cfg(test)]
    tests::grammar::test_grammar()
}
