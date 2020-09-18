use rust_htslib::{bam, bam::Read};
use structopt::StructOpt;
// CLI tutorial book
// https://rust-cli.github.io/book/tutorial/index.html
#[derive(StructOpt)]
struct Cli {
    #[structopt(parse(from_os_str))]
    infile: std::path::PathBuf,
    #[structopt(default_value = "0", short = "a", long = "above")]
    above: i64,
    #[structopt(short = "b", long = "below")]
    below: Option<i64>,
    #[structopt(short, long)]
    summary: bool
    //#[structopt(short, long)]
    //input: bam::Reader::from_path()
    //path: bam::Reader::from_path()

}



fn main() {

    let args = Cli::from_args();

    //let bam_path = &"../../test_data/yw-3LW-aH3K27me3-experiment-20-Wing-30-Std-rep1-Sup_dm6_trim_q5_dupsRemoved.bam";
    let bam_path = &args.infile;
    let mut bam = bam::Reader::from_path(bam_path).unwrap();
    let header = bam::Header::from_template(bam.header());
    let mut out = bam::Writer::from_path(&"examples/out.bam", &header, bam::Format::BAM).unwrap();

    if args.summary == true {
        let mut init = true; // tracks whether state is first read or not
        let mut min_val: i64 = 0;
        let mut max_val: i64 = 0;
        let mut mean_val: i64 = 0;
        let mut n_reads: i64 = 0;

        for r in bam.records() {
            let record = r.unwrap();
            let insert_size = record.insert_size().abs();

            // initialize & count min/max
            // at each iteration
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

            // compute rolling mean
            n_reads += 1;
            mean_val = mean_val + (insert_size - mean_val) / n_reads;

        }

        println!("min: {}", min_val);
        println!("max: {}", max_val);
        println!("mean: {}", mean_val);
        println!("n: {}", n_reads);

        return();
    }

    for r in bam.records() {
        let record = r.unwrap();
        // negative insert sizes come from reverse-strand alignment
        // so, take absolute value of size for filtering
        let insert_size = record.insert_size().abs();

        // TODO:
        // args.below needs a NONE default value,
        // but if set, filter reads in a range above:below
        // How to implement this check?
        // Current implementation is correct, but:
        // LEARN:
        // Is there better idiomatic Rust approach?

        // TODO: TESTING:
        // Will short circuiting work? like so:
        // insert_size >= args.above || args.below.is_some() && insert_size <= args.below.unwrap()

        // Doesn't work:
        // short circuits on first arg since false (duh).
        //if insert_size >= args.above || args.below.is_some() && insert_size <= args.below.unwrap() {
        //    out.write(&record).unwrap();
        //}

        if args.below.is_some() {
            if insert_size <= args.below.unwrap() && insert_size >= args.above {
                out.write(&record).unwrap();
            }
        } else {
            if insert_size >= args.above {
                out.write(&record).unwrap();
            }
        }

    }

}
