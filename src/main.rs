use rust_htslib::{bam, bam::Read};
use structopt::StructOpt;
// CLI tutorial book
// https://rust-cli.github.io/book/tutorial/index.html
#[derive(StructOpt)]
struct Cli {
    #[structopt(default_value = "0", short = "a", long = "above")]
    above: i64,
    #[structopt(default_value = "1000", short = "b", long = "below")]
    below: i64,
    #[structopt(short, long)]
    size: bool
    //path: bam::Reader::from_path()

}



fn main() {
    let bam_path = &"../../test_data/yw-3LW-aH3K27me3-experiment-20-Wing-30-Std-rep1-Sup_dm6_trim_q5_dupsRemoved.bam";
    let mut bam = bam::Reader::from_path(bam_path).unwrap();
    let header = bam::Header::from_template(bam.header());
    let mut out = bam::Writer::from_path(&"examples/out.bam", &header, bam::Format::BAM).unwrap();

    let args = Cli::from_args();

    if args.size == true {
        let mut init = true;
        let mut min_val: i64 = 0;
        let mut max_val: i64 = 0;
        for r in bam.records() {
            let record = r.unwrap();
            let insert_size = record.insert_size().abs();

            if init == true {
                min_val = insert_size;
                max_val = insert_size;
                init = false;
            } else {
                if insert_size < min_val {
                    min_val = insert_size;
                }
                if insert_size > max_val {
                    max_val = insert_size;
                }
               
            }

        }
        println!("min_val: {}", min_val);
        println!("max_val: {}", max_val);
        return();
    }

    for r in bam.records() {
        let record = r.unwrap();
        // negative insert sizes come from reverse-strand alignment
        // so, take absolute value of size for filtering
        if record.insert_size().abs() <= args.below {
            out.write(&record).unwrap();
           
        }

        if record.insert_size().abs() >= args.above {
            out.write(&record).unwrap();

        }

    }

}
