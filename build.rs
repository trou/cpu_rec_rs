#[allow(dead_code)]
mod corpus {
    include!("src/corpus.rs");
}

fn main() {
    // load default corpus
    let default = corpus::load_corpus("cpu_rec_corpus/*.corpus").unwrap();
    println!("cargo:rerun-if-changed=cpu_rec_corpus");

    // serialize to bytes
    let bytes = postcard::to_stdvec(&default).unwrap();

    // compress
    let bytes = lz4_flex::compress(&bytes);

    // output path to target build folder
    let mut outfile = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    outfile.push("default.pc");

    // write to file
    std::fs::write(outfile, bytes).unwrap();
}
