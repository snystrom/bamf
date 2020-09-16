use rust_htslib::{bam, bam::Read};
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
    #[structopt(short = "a", long = "above")]
    a: a,
    #[structopt(short = "b", long = "below")]
    b: b,
    path: bam::Reader::from_path()

}



fn main() {
    let bam_path = &"../../test_data/yw-3LW-aH3K27me3-experiment-20-Wing-30-Std-rep1-Sup_dm6_trim_q5_dupsRemoved.bam";
    let mut bam = bam::Reader::from_path(bam_path).unwrap();
    let header = bam::Header::from_template(bam.header());
    let mut out = bam::Writer::from_path(&"examples/out.bam", &header, bam::Format::BAM).unwrap();

    for r in bam.records() {
        let record = r.unwrap();
        // negative insert sizes come from reverse-strand alignment
        // so, take absolute value of size for filtering
        if record.insert_size().abs() < 120 {
            out.write(&record).unwrap();
           
        }

    }

}
