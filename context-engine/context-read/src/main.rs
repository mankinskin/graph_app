//pub mod bundle;
pub mod complement;
pub mod context;
pub mod expansion;
//pub mod overlap;
pub mod sequence;
//#[cfg(test)]
//mod tests;

#[cfg(test)]
pub mod tests;

fn main() {
    #[cfg(test)]
    tests::grammar::test_grammar()
}
