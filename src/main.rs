mod corpus;
use crate::corpus::CorpusStats;
use anyhow::{Context, Result};
use glob::glob;
use std::str::FromStr;
use std::string::String;
use log::{LevelFilter, info};
use simple_logger;

fn main() -> Result<()> {
    simple_logger::init_with_level(log::Level::Debug).unwrap();
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
            )
        })
        .collect();
    println!("{}", corpus_stats.len());
    //println!("{:?}", corpus_stats);
    Ok(())
}
