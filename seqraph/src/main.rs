pub mod direction;
pub mod search;
pub mod join;
pub mod insert;
//pub mod read;

#[cfg(test)]
pub mod tests;

fn main() {
    #[cfg(test)]
    tests::grammar::test_grammar()
}
