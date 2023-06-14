use anyhow::{Context, Result, Ok, Error};
use log::debug;
use std::collections::HashMap;
use std::fmt::Debug;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use glob::glob;
use std::str::FromStr;
use std::string::String;

#[derive(Debug)]
pub struct CorpusStats {
    pub arch: String,
    bigrams_freq: HashMap<(u8, u8), f64>,
    trigrams_freq: HashMap<(u8, u8, u8), f64>,
    bg_base_freq: f64,
    tg_base_freq: f64,
}

pub fn load_corpus(path : &String) -> Result<Vec<CorpusStats>, Error> {
    let corpus_entries = glob(path)
        .with_context(|| "Could not find \"cpu_rec_corpus\" directory.")?
        .map(|p| p.unwrap());

    let res : Result<Vec<CorpusStats>, _> = corpus_entries
        .map(|p| {
            let arch_name =
                String::from_str(p.file_name().unwrap().to_os_string().to_str().unwrap())
                    .unwrap()
                    .replace(".corpus", "");
            let mut data = Vec::<u8>::new();
            debug!("Loading {} for arch {}", p.display(), arch_name);
            load_file(&p, &mut data)?;
            Ok(CorpusStats::new(arch_name, &mut data, 0.01))
        })
        .collect();
    res
}

pub fn load_file(file: &Path, data : &mut Vec<u8>) -> Result<(), Error> {
    File::open(file).unwrap().read_to_end(data).with_context(|| "Could not read file")?;
    Ok(())
}
impl CorpusStats {
    pub fn new(arch: String, data: &Vec<u8>, base_count: f64) -> Self {
        let mut bg: HashMap<(u8, u8), f64> = HashMap::new();
        let mut tg = HashMap::new();

        /*
        Duplicate code to be able to use tuples, for optimization
        */
        for w in data.windows(2) {
            let b = (w[0], w[1]);
            bg.entry(b)
                .and_modify(|count| *count += 1.0)
                .or_insert(1.0 + base_count);
        }
        for w in data.windows(3) {
            let b = (w[0], w[1], w[2]);
            tg.entry(b)
                .and_modify(|count| *count += 1.0)
                .or_insert(1.0 + base_count);
        }
        debug!(
            "{}: {} bytes, {} bigrams, {} trigrams",
            arch,
            data.len(),
            bg.len(),
            tg.len()
        );
        let bi_qtotal: f64 =
            (base_count * ((u32::pow(256, 2) - bg.len() as u32) as f64)) + bg.values().sum::<f64>();
        debug!("{} bigrams Qtotal: {}", arch, bi_qtotal);
        let tri_qtotal: f64 =
            (base_count * ((u32::pow(256, 3) - tg.len() as u32) as f64)) + tg.values().sum::<f64>();
        debug!("{} trigrams Qtotal: {}", arch, tri_qtotal);

        // Update counts to frequencies
        let bg_freq = bg.into_iter().map(|(k, v)| (k, (v / bi_qtotal))).collect();
        let tg_freq = tg.into_iter().map(|(k, v)| (k, (v / tri_qtotal))).collect();
        CorpusStats {
            arch,
            bigrams_freq: bg_freq,
            trigrams_freq: tg_freq,
            bg_base_freq: base_count / bi_qtotal,
            tg_base_freq: base_count / tri_qtotal,
        }
    }

    pub fn compute_kl(&self, q: &Self) -> (f64, f64) {
        let mut kld_bg = 0.0;
        for (bg, f) in &self.bigrams_freq {
            if *f != 0.0 {
                kld_bg += f * (f / q.bigrams_freq.get(&bg).unwrap_or(&q.bg_base_freq)).ln();
            }
        }
        let mut kld_tg = 0.0;
        for (tg, f) in &self.trigrams_freq {
            if *f != 0.0 {
                kld_tg += f * (f / q.trigrams_freq.get(&tg).unwrap_or(&q.tg_base_freq)).ln();
            }
        }
        (kld_bg, kld_tg)
    }
}
