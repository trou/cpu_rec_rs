mod corpus;
use crate::corpus::CorpusStats;
use anyhow::{Context, Result};
use glob::glob;
use std::str::FromStr;
use std::string::String;
use log::{info};
use simple_logger;

fn main() -> Result<()> {
    simple_logger::init_with_level(log::Level::Info).unwrap();
    info!("Test");
    let corpus_entries = glob("cpu_rec_corpus/*.corpus")
        .with_context(|| "Could not find \"cpu_rec_corpus\" directory.")?
        .map(|p| p.unwrap());
    let corpus_stats: Vec<CorpusStats> = corpus_entries
        .map(|p| {
            let arch_name = String::from_str(p.file_name().unwrap().to_os_string().to_str().unwrap()).unwrap().replace(".corpus", "");
            CorpusStats::new(
                arch_name,
                &p,
                0.01
            )
        })
        .collect();
    println!("{}", corpus_stats.len());
    let target = CorpusStats::new(String::from_str("target")?, std::path::Path::new("/bin/true"), 0.0);
    let results : Vec<(&String, (f64, f64))> = corpus_stats.iter().map(|c| (&c.arch, target.compute_kl(&c))).collect();
    let mut sorted_m2 : Vec<(&String, f64)> = results.iter().map(|r| (r.0, r.1.0)).collect();
    sorted_m2.sort_unstable_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    info!("Results 2-gram: {:?}", sorted_m2) ;
    let mut sorted_m3 : Vec<(&String, f64)> = results.iter().map(|r| (r.0, r.1.1)).collect();
    sorted_m3.sort_unstable_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    info!("Results 3-gram: {:?}", sorted_m3) ;
    Ok(())
}
