use itertools::Itertools;
use rand::{
    seq::IteratorRandom, SeedableRng,
};
use rand_distr::{
    Distribution,
    Normal,
};
use crate::HypergraphRef;
use std::{panic::{
    catch_unwind,
    AssertUnwindSafe,
}, sync::{Arc, Mutex, RwLock}, collections::HashMap};

lazy_static::lazy_static! {
    static ref PANIC_INFO: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
}
pub fn gen_graph() -> HypergraphRef<char> {
    let mut graph = Some(HypergraphRef::default());
    let fuzz_len = 100000;
    let len_distr: Normal<f32> = Normal::new(20.0, 4.0).unwrap();
    let mut rng = rand::rngs::StdRng::seed_from_u64(42);
    let mut panics = HashMap::new();
    let input_distr = "abcd ".chars().collect_vec();
    let mut panic_count = 0;
    for i in 0..fuzz_len {
        let mut length = 0;
        while length < 1 {
            length = len_distr.sample(&mut rng) as usize;
        }
        let input = input_distr.iter().choose_multiple(&mut rng, length).into_iter().collect::<String>();
        let prev_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(|info| {
            *PANIC_INFO.lock().unwrap() = info.location().map(|loc| format!("{}", loc));
        }));
        match catch_unwind(|| {
            graph.clone().unwrap().read_sequence(input.chars())
        }) {
            Ok(_) => {},
            Err(_) => {
                std::panic::set_hook(prev_hook);
                let inner = graph.take().unwrap();
                graph = Some(HypergraphRef::from(Arc::try_unwrap(inner.0).unwrap().into_inner().unwrap_or_else(|p| p.into_inner())));
                let msg = PANIC_INFO.lock().unwrap().take().unwrap();
                panics.entry(msg).and_modify(|instances: &mut Vec<_>| instances.push((i, input.clone())))
                    .or_insert_with(|| vec![(i, input)]);
                panic_count += 1;
            },
        }
    }
    if panics.len() > 0 {
        print!("\n\n");
        for (err, instances) in panics {
            println!("\nPanic at {}: {}", err, instances.len() as f32/panic_count as f32);
        }
        panic!("Panics: {}/{}", panic_count, fuzz_len);
    }
    graph.unwrap()
}