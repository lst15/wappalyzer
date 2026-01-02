extern crate url;

use futures::future::join_all;
use std::env;
use std::io::{self, Read};
use std::time::Instant;
use url::Url;
use wappalyzer::scan;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    let mut urls = vec![];
    if args.len() == 1 {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        urls.extend(strings_to_urls(buffer));
    } else {
        urls.push(Url::parse(&String::from(&args[1]))?);
    }

    let benchmark_enabled = env::var("WAPPALYZER_BENCHMARK").is_ok();
    let total_urls = urls.len();
    let benchmark_start = if benchmark_enabled {
        Some(Instant::now())
    } else {
        None
    };

    let futures = urls.into_iter().map(scan).collect::<Vec<_>>();
    let results = join_all(futures).await;
    for res in results {
        if let Ok(output) = serde_json::to_string(&res) {
            println!("{}", output);
        }
    }

    if let Some(start) = benchmark_start {
        let elapsed = start.elapsed();
        let elapsed_secs = elapsed.as_secs_f64().max(f64::EPSILON);
        let rate = total_urls as f64 / elapsed_secs;
        eprintln!(
            "Benchmark: scanned {} urls in {:.2?} ({:.2} urls/s)",
            total_urls, elapsed, rate
        );
    }
    Ok(())
}

fn strings_to_urls(domains: String) -> Vec<Url> {
    domains
        .split_terminator('\n')
        .map(|s| Url::parse(s))
        .filter_map(Result::ok)
        .collect()
}
