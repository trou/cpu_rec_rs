mod corpus;
use crate::corpus::{load_corpus, load_file, CorpusStats};
use anyhow::{Error, Result};
use clap::{arg, Arg, ArgAction};
use log::{debug, info};
use std::str::FromStr;
use std::string::String;

fn determine(r2: &[(String, f64)], r3: &[(String, f64)]) -> Option<String> {
    /* Bigrams and trigrams disagree or "special" invalid arch => no result */
    if (r2[0].0 != r3[0].0) || r2[0].0.starts_with('_') {
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

fn predict(corpus_stats: &Vec<CorpusStats>, target: &CorpusStats) -> Result<Option<String>, Error> {
    let mut results_m2 = Vec::<(String, f64)>::with_capacity(corpus_stats.len());
    let mut results_m3 = Vec::<(String, f64)>::with_capacity(corpus_stats.len());
    for c in corpus_stats {
        let r = target.compute_kl(c);
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
    let app = clap::Command::new("cpu_rec_rs")
        .version(env!("CARGO_PKG_VERSION"))
        .propagate_version(true)
        .author("Raphaël Rigo <devel@syscall.eu>")
        .about("Identifies CPU architectures in binaries")
        .arg(arg!(--corpus <corpus_dir>).default_value("cpu_rec_corpus"))
        .arg(arg!(-d - -debug))
        .arg(arg!(-v - -verbose))
        .arg(
            Arg::new("files")
                .action(ArgAction::Append)
                .value_parser(clap::builder::NonEmptyStringValueParser::new())
                .required(true),
        );

    let args = app.get_matches();

    let level = if args.get_flag("debug") {
        log::Level::Debug
    } else if args.get_flag("verbose") {
        log::Level::Info
    } else {
        log::Level::Warn
    };
    simple_logger::init_with_level(level)?;
    let corpus_dir: String = args.get_one::<String>("corpus").unwrap().to_owned() + "/*.corpus";
    info!("Loading corpus from {}", corpus_dir);
    let corpus_stats = load_corpus(&corpus_dir)?;

    info!("Corpus size: {}", corpus_stats.len());

    for file in args.get_many::<String>("files").unwrap() {
        print!("{} ", file);
        let mut file_data = Vec::<u8>::new();
        load_file(std::path::Path::new(file), &mut file_data)?;

        let target = CorpusStats::new(String::from_str("target")?, &file_data, 0.0);
        println!(
            "{}",
            predict(&corpus_stats, &target)?.unwrap_or_else(|| "Unknown".to_string())
        );
    }
    Ok(())
}
