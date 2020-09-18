use rust_htslib::{bam, bam::Read};
use structopt::StructOpt;
// CLI tutorial book
// https://rust-cli.github.io/book/tutorial/index.html
//
// Advice for using enum + struct for building subcommands:
// https://stackoverflow.com/a/61351721

#[derive(StructOpt, Debug)]
struct Cli {
    #[structopt(subcommand)]
    commands: Option<Bamf>
}
#[derive(StructOpt, Debug)]
struct FilterOpts {
    #[structopt(parse(from_os_str))]
    infile: std::path::PathBuf,
    #[structopt(default_value = "0", short = "a", long = "above")]
    above: i64,
    #[structopt(short = "b", long = "below")]
    below: Option<i64>,
}

#[derive(StructOpt, Debug)]
struct SummaryOpts {
    #[structopt(parse(from_os_str))]
    infile: std::path::PathBuf,
    #[structopt(short, long)]
    summary: bool

}

#[derive(StructOpt, Debug)]
#[structopt(name = "bamf", about = "easier to grok than awk")]
enum Bamf {
    #[structopt(name = "filter")]
    Filter (FilterOpts),

    #[structopt(name = "summary")]
    Summary (SummaryOpts),

}

fn filter(args: &FilterOpts, record: &bam::record::Record, out: &mut bam::Writer) {
    //let record = r.unwrap();
    //
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
            out.write(record).unwrap();
        }
    } else {
        if insert_size >= args.above {
            out.write(record).unwrap();
        }
    }

}

fn summary(bam: &mut bam::Reader) {
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

fn create_infile_bam_connection(path: &std::path::PathBuf) -> bam::Reader {
    return bam::Reader::from_path(path).unwrap()
}

fn create_stdout_bam_connection(infile: &bam::Reader) -> bam::Writer {
    let header = bam::Header::from_template(infile.header());
    let out = bam::Writer::from_stdout(&header, bam::Format::BAM).unwrap();
    return out;
}

fn main() {

    let cli = Cli::from_args();

    if let Some(subcommand) = cli.commands{
        match subcommand {
            Bamf::Filter(args) => {
                //println!("handle Add:  {:?}", args);

                let mut bam = create_infile_bam_connection(&args.infile);
                let mut out = create_stdout_bam_connection(&bam);

                for r in bam.records() {
                    let record = r.unwrap();
                    filter(&args, &record, &mut out);
                }

            },
            Bamf::Summary(args) => {
                //println!("handle Commit: {:?}", args);
                let mut bam = create_infile_bam_connection(&args.infile);
                summary(&mut bam)
            }
        }
    }


}

