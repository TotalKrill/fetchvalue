use std::time::{Duration, Instant};

use fetchvalue::FetchValue;
use serde::Deserialize;

#[derive(Deserialize, Debug, Default, Clone)]
struct CatFact {
    fact: String,
    length: usize,
}

fn main() {
    println!("hello");
    // we create a fetcher that will fetch the value using a get URL, and each time the value is accessed through .value()
    let mut fetcher = FetchValue::<CatFact>::new("https://catfact.ninja/fact")
        .starting_value(CatFact {
            fact: "we know nothing of cats yet! the first fetch has not gone through".into(),
            length: 4,
        })
        .max_rate(Duration::from_secs(3));

    // start now tells the fetcher to issue the first fetch request right away, it can be run multiple times, but until the value
    // is has been recieved, it will show the old value...
    let now = Instant::now();
    std::thread::sleep_ms(1000);

    let v = fetcher.value();

    // the value will probably not be updated here, but in a few millis, until then, the starting value we defined will be used
    println!("{}\t{}", Instant::elapsed(&now).as_millis(), v.fact);

    // the first update in this loop should then happen after second 4 ( 3 + sleeping 1 + whatever time it takes to fetch value twice)
    loop {
        let v = fetcher.value();
        println!("{}\t{}", Instant::elapsed(&now).as_millis(), v.fact);
        std::thread::sleep_ms(20);
    }
}
