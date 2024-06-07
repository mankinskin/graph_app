use itertools::Itertools;
use ngram::NGram;
use tap::Tap;

use seqraph::HashMap;

pub fn ngram_set(s: String) -> Vec<HashMap<String, usize>>
{
    let slen: usize = s.len();
    let N_MAX: usize = 10;
    (1..N_MAX)
        .into_iter()
        .map(|n| {
            let ngrams = s.chars().ngrams(n);
            ngrams
                .fold((0, HashMap::default()), |(mut len, mut set), x| {
                    let x = String::from_iter(x);
                    //println!("{}", x);
                    set.entry(x).and_modify(|v| *v += 1).or_insert_with(|| {
                        len += 1;
                        1 as usize
                    });
                    (len, set)
                })
                .tap(|(len, _)| println!("Finished counting n={}: {}", n, len))
                .1
        })
        .collect()
}

//#[test]
pub fn test_ngrams()
{
    //let file_path = "./corpus/eng_news_2020_100K/eng_news_2020_100K-sentences.txt";
    //let corpus: String = read_corpus(file_path);
    //println!("Finished reading {}", file_path);
    //
    let corpus = crate::OTTOS_MOPS_CORPUS;
    let mut total_counts: HashMap<String, usize> = HashMap::default();
    for s in corpus
    {
        for (n, counts) in ngram_set(s.to_string()).into_iter().enumerate()
        {
            for (gr, c) in counts
            {
                total_counts.entry(gr).and_modify(|t| *t += c).or_insert(c);
            }
        }
    }
    println!("All n counted");
    println!("[");
    for (g, c) in total_counts
        .iter()
        .filter(|(_, c)| **c != 1)
        .sorted_by_key(|(_, c)| **c)
    {
        println!("    \"{}\": {}", g, c);
    }
    println!("]");

    let C_MAX: usize = 20;
    let hist = total_counts.into_iter().fold(
        HashMap::<usize, usize>::default(),
        |mut hist, (gr, c)| {
            hist.entry(c).and_modify(|t| *t += 1).or_insert(1);
            hist
        },
    );
    for (c, n) in hist.into_iter().sorted()
    {
        println!("{}: {}", c, n);
    }
}
