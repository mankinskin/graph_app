pub mod insert;
pub mod search;
//pub mod read;

#[cfg(test)]
pub mod tests;

fn main() {
    #[cfg(test)]
    tests::grammar::test_grammar()
}
