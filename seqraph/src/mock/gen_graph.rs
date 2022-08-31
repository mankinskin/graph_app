use itertools::Itertools;
use rand::{
    rngs::StdRng,
    seq::IteratorRandom, SeedableRng, RngCore,
};
use rand_distr::{
    Distribution,
    Normal,
};
use crate::HypergraphRef;
use std::{
    panic::{
        catch_unwind,
    },
    sync::{Arc, Mutex},
    collections::HashMap,
};
use std::time::{
    Duration,
};

lazy_static::lazy_static! {
    static ref PANIC_INFO: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
}
#[allow(unused)]
pub fn gen_graph() -> Result<HypergraphRef<char>, HypergraphRef<char>> {
    let batch_size = 10;
    let batch_count = 1;
    let fuzz_len = batch_size * batch_count;
    let mean_length = 50;
    let len_distr: Normal<f32> = Normal::new(mean_length as f32, 4.0).unwrap();
    let mut rng = StdRng::seed_from_u64(1);
    //let mut rng = StdRng::from_seed([
    //    63, 104, 23, 252, 201, 228, 199, 24, 104, 10,
    //    207, 224, 132, 156, 25, 149, 104, 185, 78, 117,
    //    231, 247, 115, 133, 119, 203, 168, 44, 31, 31, 54, 42
    //]);
    let mut panics = HashMap::new();
    let input_distr = "abcdefghi ".chars().collect_vec();
    let mut panic_count = 0;
    let pb = indicatif::ProgressBar::with_draw_target(
        fuzz_len,
        indicatif::ProgressDrawTarget::stdout(),
    );
    let prev_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|info| {
        *PANIC_INFO.lock().unwrap() = info.location().map(|loc| format!("{}", loc));
    }));
    let fuzz_progress = indicatif::ProgressBar::with_draw_target(
        batch_count,
        indicatif::ProgressDrawTarget::stdout(),
    );
    let mut len_histo = histo::Histogram::with_buckets(10);
    let mut time_histo = histo::Histogram::with_buckets(10);
    let mut graph = None;
    let mut total_time = Duration::ZERO;
    for _ in 0..batch_count {
        graph = Some(HypergraphRef::default());
        fuzz_progress.inc(1);
        let batch_progress = indicatif::ProgressBar::with_draw_target(
            batch_size,
            indicatif::ProgressDrawTarget::stdout(),
        );
        let mut successful = 0;
        let mut batch_time = Duration::ZERO;
        for i in 0..batch_size {
            //print!("{},", i);
            let mut seed = <StdRng as SeedableRng>::Seed::default();
            rng.try_fill_bytes(seed.as_mut()).unwrap();
            batch_progress.inc(1);
            let mut length = 0;
            while length < 1 {
                length = len_distr.sample(&mut rng) as usize;
            }
            len_histo.add(length as u64);
            let input = (0..length).map(|_| *input_distr.iter().choose(&mut rng).unwrap()).collect::<String>();
            let now = std::time::Instant::now();
            match catch_unwind(|| {
                graph.clone().unwrap().read_sequence(input.chars());
            }) {
                Ok(_) => {
                    batch_time += now.elapsed();
                    successful += 1;
                },
                Err(_) => {
                    let inner = graph.take().unwrap();
                    graph = Some(HypergraphRef::from(
                        Arc::try_unwrap(inner.0).unwrap()
                        .into_inner()
                        .unwrap_or_else(|p| p.into_inner()))
                    );
                    let msg = PANIC_INFO.lock().unwrap().take().unwrap();
                    let new = (i, input, seed);
                    panics.entry(msg).and_modify(|instances: &mut Vec<_>|
                        instances.push(new.clone())
                    )
                    .or_insert_with(|| vec![new]);
                },
            }
        }
        panic_count += batch_size - successful;
        if successful > 0 {
            time_histo.add(batch_time.as_millis() as u64/successful);
            total_time += batch_time;
        }
    }
    std::panic::set_hook(prev_hook);
    pb.finish_and_clear();
    if panics.is_empty() {
        Ok(graph.unwrap())
    } else {
        println!("\nPanic locations:");
        for (err, instances) in panics.iter() {
            let percent = ((instances.len() as f32/panic_count as f32) * 100.0) as u32;
            println!("\nPanic at {}: {}%", err, percent);
        }
        println!("Panics: {}/{}", panic_count, fuzz_len);
        println!("Seeds: {:?}", panics.into_values().flat_map(|inst| inst.into_iter().map(|inst| inst.2)).collect_vec());
        println!("lengths:\n{}", len_histo);
        println!("times:\n{}", time_histo);
        println!("Total time:\n{} ms", total_time.as_millis());
        println!("Average character rate:\n{} chars/sec", if total_time.as_millis() > 0 {
            (1000 * mean_length * batch_count)/total_time.as_millis() as u64
        } else {
            0
        });
        Err(graph.unwrap())
    }
}