mod corpus;
use crate::corpus::{CorpusStats, load_file, load_corpus};
use anyhow::{Context, Result, Error};
use clap::{Arg, ArgAction, arg};
use log::{debug, info};
use simple_logger;
use std::str::FromStr;
use std::string::String;

fn determine<'a>(r2: &'a Vec<(String, f64)>, r3: &Vec<(String, f64)>) -> Option<String> {
    /* Bigrams and trigrams disagree or "special" invalid arch => no result */
    if (r2[0].0 != r3[0].0) || r2[0].0.chars().next().unwrap() == '_' {
        return None;
    }
    let res = &r2[0].0;
    /* Special heuristics */
    if (res == "Ocaml" && r2[0].1 > 1.0) || (res == "IA-64" && r2[0].1 > 3.0) {
        return None;
    }
    return Some(r2[0].0.clone());
    /* TODO:
    elif res == 'PIC24':
            # PIC24 code has a 24-bit instruction set. In our corpus it is encoded in 32-bit words,
            # therefore every four byte is 0x00.
            zero = [True, True, True, True]
            for idx in range(len(d) // 4):
                zero = [zero[i] and d[4 * idx + i] == 0 for i in range(4)]
                if True not in zero:
                    return None
        return res
     */
}

fn predict<'a>(corpus_stats : &Vec<CorpusStats>, target : &CorpusStats) -> Result<Option<String>, Error> {
    let mut results_m2 = Vec::<(String, f64)>::with_capacity(corpus_stats.len());
    let mut results_m3 = Vec::<(String, f64)>::with_capacity(corpus_stats.len());
    for c in corpus_stats {
        let r = target.compute_kl(&c);
        results_m2.push((c.arch.clone(), r.0));
        results_m3.push((c.arch.clone(), r.1));
    }
    results_m2.sort_unstable_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    debug!("Results 2-gram: {:?}", results_m2);
    results_m3.sort_unstable_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    debug!("Results 3-gram: {:?}", results_m3);
    Ok(determine(&results_m2, &results_m3))
}

fn main() -> Result<()> {

    let mut app = clap::Command::new("cpu_rec_rs")
        .version(env!("CARGO_PKG_VERSION"))
        .propagate_version(true)
        .author("Raphaël Rigo <devel@syscall.eu>")
        .about("Identifies CPU architectures in binaries")
        .arg(arg!(--corpus <corpus_dir>).default_value("cpu_rec_corpus"))
        .arg(arg!(--debug))
        .arg(
            Arg::new("files")
                .action(ArgAction::Append)
                .value_parser(clap::builder::NonEmptyStringValueParser::new())
                .required(true),
        );

    // Parse args
    let matches = app.get_matches_mut();

    if matches.get_flag("debug") {
        simple_logger::init_with_level(log::Level::Debug).unwrap();
    } else {
        simple_logger::init_with_level(log::Level::Info).unwrap();
    }
    let corpus_dir : String = matches.get_one::<String>("corpus").unwrap().to_owned()+"/*.corpus";
    let corpus_stats = load_corpus(&corpus_dir)?;

    info!("Corpus size: {}", corpus_stats.len());

    // TODO iterate
    for file in matches.get_many::<String>("files").unwrap() {
    print!("{} ", file);
    let mut file_data = Vec::<u8>::new();
    load_file(&std::path::Path::new(file), &mut file_data)?;

    let target = CorpusStats::new(String::from_str("target")?, &file_data, 0.0);
    println!("{:?}", predict(&corpus_stats, &target)?);
    }
    Ok(())
}
