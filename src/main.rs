/*
    Copyright 2023 - Raphaël Rigo

    Licensed under the Apache License, Version 2.0 (the "License");
    you may not use this file except in compliance with the License.
    You may obtain a copy of the License at

        http://www.apache.org/licenses/LICENSE-2.0

    Unless required by applicable law or agreed to in writing, software
    distributed under the License is distributed on an "AS IS" BASIS,
    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
    See the License for the specific language governing permissions and
    limitations under the License.
*/
mod corpus;
use crate::corpus::{load_corpus, CorpusStats};
use anyhow::{Context, Error, Result};
use clap::{arg, Arg, ArgAction};
use log::{debug, info};
use std::cmp::min;
use std::path::Path;
use std::str::FromStr;
use std::string::String;

#[derive(Clone)]
struct DetectionResult {
    file: String,
    arch: String,
    range: String,
}

// Apply final heuristics to guess the arch
fn determine(r2: &KlRes, r3: &KlRes) -> Option<String> {
    /* Bigrams and trigrams disagree or "special" invalid arch => no result */
    if (r2.arch != r3.arch) || r2.arch.starts_with('_') {
        return None;
    }
    let res = &r2.arch;
    /* Special heuristics */
    if (res == "OCaml" && r2.div > 1.0)
        || (res == "xmox_xs2a" && r2.div > 3.0)
        || (res == "IA-64" && r2.div > 3.0)
    {
        debug!("{}, probably a false positive", res);
        return None;
    }
    Some(res.clone())
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

#[derive(Debug)]
struct KlRes {
    arch: String,
    div: f32,
}

fn predict(corpus_stats: &Vec<CorpusStats>, target: &CorpusStats) -> Result<Option<String>, Error> {
    let mut results_m2 = Vec::<KlRes>::with_capacity(corpus_stats.len());
    let mut results_m3 = Vec::<KlRes>::with_capacity(corpus_stats.len());

    // Build a vec of results for bigrams and trigrams, for easier processing
    for c in corpus_stats {
        let r = target.compute_kl(c);
        results_m2.push(KlRes {
            arch: c.arch.clone(),
            div: r.bigrams,
        });
        results_m3.push(KlRes {
            arch: c.arch.clone(),
            div: r.trigrams,
        });
    }

    // Sort
    results_m2.sort_unstable_by(|a, b| a.div.partial_cmp(&b.div).unwrap());
    debug!("Results 2-gram: {:?}", &results_m2[0..2]);
    results_m3.sort_unstable_by(|a, b| a.div.partial_cmp(&b.div).unwrap());
    debug!("Results 3-gram: {:?}", &results_m3[0..2]);

    // Guess
    let res = determine(&results_m2[0], &results_m3[0]);
    debug!("Result: {:?}", res);
    Ok(res)
}

/* Try to guess the architecture of `file_data`:
    * first by analyzing the whole buffer
    * then by applying a sliding window and analyzing it, making it smaller and smaller until a result is found
  The function returns a vec containing the results
*/
fn guess_with_windows(
    corpus_stats: &Vec<CorpusStats>,
    file_data: &Vec<u8>,
    filename: &str,
) -> Result<Vec<DetectionResult>, Error> {
    let mut res = Vec::<DetectionResult>::new();

    let target = CorpusStats::new(String::from_str("target")?, file_data, 0.0);
    let res_full = predict(corpus_stats, &target)?;

    // If the whole file data gives a result, return it
    if let Some(r) = res_full {
        res.push(DetectionResult {
            arch: r,
            file: filename.to_string(),
            range: "Whole file".to_string(),
        });
        return Ok(res);
    }

    // Heuristic depending on file size, the number is actually half the window
    // size
    let mut window = match file_data.len() {
        0x20001..=0x100000 => 0x800,
        0x8001..=0x20000 => 0x400,
        0x1001..=0x8000 => 0x200,
        0..=0x1000 => 0x100,
        _ => (file_data.len() / 100) & 0xFFFFF000,
    };

    let mut ok = false;
    while window >= 0x40 && !ok {
        /*
            Store the current guess, in order to update the range while the arch is
            the same over the consecutive windows
        */
        struct Guess {
            arch: Option<String>,
            range: [usize; 2],
        }

        let mut cur_guess: Guess = Guess {
            arch: None,
            range: [0, 0],
        };

        info!("{}: window_size : 0x{:x} ", filename, window * 2);
        for start in (0..file_data.len()).step_by(window) {
            let end = min(file_data.len(), start + window * 2);

            debug!("{}: range 0x{:x}-0x{:x}", filename, start, end);
            let win_stats =
                CorpusStats::new("target".to_string(), &file_data[start..end].to_vec(), 0.0);
            let win_res = predict(corpus_stats, &win_stats)?;

            // Should we add the previous guess to the result ?  yes if it's
            // either unknown (None) or different from the new one
            let do_push = match &win_res {
                Some(wres) => !cur_guess.arch.as_ref().is_some_and(|a| a == wres),
                _ => true,
            };
            if do_push {
                // push the detected arch to the results if it's known and
                // covers more than one window
                if cur_guess.arch.is_some()
                    && (cur_guess.range[1] - cur_guess.range[0]) > window * 2
                {
                    res.push(DetectionResult {
                        file: filename.to_string(),
                        arch: cur_guess.arch.unwrap(),
                        range: format!("0x{:x}-0x{:x}", cur_guess.range[0], cur_guess.range[1]),
                    });
                }
                // Update the current guess
                cur_guess.arch = win_res;
                cur_guess.range[0] = start;
                cur_guess.range[1] = end;
            } else {
                // Same arch: update the end of the range
                cur_guess.range[1] = end;
            }
        }

        // No result: try a smaller window, else return
        if res.is_empty() {
            window /= 2;
        } else {
            ok = true;
        }
    }

    Ok(res)
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

    let corpus_dir = args.get_one::<String>("corpus").unwrap().to_owned();
    if !Path::new(&corpus_dir).is_dir() {
        return Err(Error::msg(format!("{} is not a valid directory", corpus_dir)));
    }
    let corpus_files: String = args.get_one::<String>("corpus").unwrap().to_owned() + "/*.corpus";
    println!("Loading corpus from {}", corpus_files);

    let corpus_stats = load_corpus(&corpus_files)?;

    info!("Corpus size: {}", corpus_stats.len());

    // Prepare output stream
    let mut out = std::io::stdout();
    let mut tablestream = tablestream::Stream::new(
        &mut out,
        vec![
            tablestream::col!(DetectionResult: .file).header("File"),
            tablestream::col!(DetectionResult: .range).header("Range"),
            tablestream::col!(DetectionResult: .arch).header("Detected Architecture"),
        ],
    );

    for file in args.get_many::<String>("files").unwrap() {
        let file_data = std::fs::read(file).with_context(|| format!("Could not open {}", file))?;

        for g in guess_with_windows(&corpus_stats, &file_data, file)? {
            tablestream.row(g)?;
        }
    }
    tablestream.finish()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_approx_eq::assert_approx_eq;
    use corpus::*;
    use rand::{RngCore, SeedableRng};

    #[test]
    fn check_stats() {
        let mut rng = rand::rngs::StdRng::seed_from_u64(2);
        let mut rand_data: [u8; 1000] = [1; 1000];
        rng.fill_bytes(&mut rand_data);

        let mut rand_target: [u8; 1000] = [1; 1000];
        rng.fill_bytes(&mut rand_target);

        let rand_stats = CorpusStats::new("rand".to_string(), &rand_data.to_vec(), 0.01);
        let target_stats = CorpusStats::new("rand".to_string(), &rand_data.to_vec(), 0.0);

        let res = target_stats.compute_kl(&rand_stats);
        assert_approx_eq!(res.bigrams, 0.49450362);
        assert_approx_eq!(res.trigrams, 5.120544);
    }
}
