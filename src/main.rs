use rust_htslib::{bam, bam::Read};
use structopt::{clap::ArgGroup, StructOpt};
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
    /// a bam file
    #[structopt(parse(from_os_str))]
    infile: std::path::PathBuf,
    /// return all fragments greater than or equal to this fragment size
    #[structopt(default_value = "0", short = "a", long = "above")]
    above: i64,
    /// return all fragments less than or equal to this fragment size
    #[structopt(short = "b", long = "below")]
    below: Option<i64>,
}

#[derive(StructOpt, Debug)]
#[structopt(group = ArgGroup::with_name("stat"))]
struct SummaryOpts {
    /// a bam file
    #[structopt(parse(from_os_str))]
    infile: std::path::PathBuf,
    /// Print minimum fragment size
    #[structopt(short = "n", long, group = "stat", conflicts_with = "summary")]
    min: bool,
    /// Print maximum fragment size
    #[structopt(short = "x", long, group = "stat", conflicts_with = "summary")]
    max: bool,
    /// Print mean fragment size
    #[structopt(short = "d", long, group = "stat", conflicts_with = "summary")]
    mean: bool,
    /// Print total read count
    #[structopt(short = "c", long, group = "stat", conflicts_with = "summary")]
    reads: bool,

}

#[derive(StructOpt, Debug)]
struct HistogramOpts {
    /// a bam file
    #[structopt(parse(from_os_str))]
    infile: std::path::PathBuf,
    /// compute distribution up to this size
    #[structopt(default_value = "1000", short = "b", long = "below")]
    below: u64
}

#[derive(StructOpt, Debug)]
struct SplitOpts {
    /// a bam file
    #[structopt(parse(from_os_str))]
    infile: std::path::PathBuf,
    /// a series of fragment sizes to split on
    #[structopt(short = "s", long = "split")]
    split: Vec<i64>,
    /// Keep fragments equal to or above this size
    #[structopt(default_value = "0", short = "a", long = "above")]
    above: i64,
    /// Keep fragments equal to or below this size
    #[structopt(short = "b", long = "below")]
    below: Option<i64>,
    /// File prefix
    #[structopt(short = "p", long = "prefix")]
    prefix: Option<String>,

}

#[derive(StructOpt, Debug)]
enum Bamf {
    /// Filter bam file to keep only fragments of given size
    #[structopt(name = "filter")]
    Filter (FilterOpts),

    /// Print fragment size summary statistics
    #[structopt(name = "stats")]
    Summary (SummaryOpts),

    /// Return counts of each fragment size in csv format
    #[structopt(name = "histogram")]
    Hist (HistogramOpts),

    /// Filter bam file into multiple bam files according to fragment size intervals
    #[structopt(name = "split")]
    Split (SplitOpts),

}

struct BamSummary {
    min: i64,
    max: i64,
    mean: i64,
    reads: i64
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

fn summary(bam: &mut bam::Reader) -> BamSummary {
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
        if init {
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

     let result = BamSummary {
        min: min_val,
        max: max_val,
        mean: mean_val,
        reads: n_reads
     };

    result
}

fn hist(bam: &mut bam::Reader, below: u64){
    let mut h = histogram::Histogram::configure()
                        .max_value(below)
                        .build()
                        .unwrap();

    for r in bam.records() {
        let insert_size = r.unwrap().insert_size().abs() as u64;

        if insert_size <= below {
            //TODO handle Err
            h.increment(insert_size)
                .expect("Cannot increment histogram counts");
            // New implementation:
        }
    }

    // write histogram to stdout
    // in csv format:
    // value,count
    println!("size,n");
    let h_iter = h.into_iter();
    for i in h_iter {
        println!("{},{}", i.value(), i.count());

    }

}


fn create_infile_bam_connection(path: &std::path::PathBuf) -> bam::Reader {
    bam::Reader::from_path(path).unwrap()
}

fn create_stdout_bam_connection(infile: &bam::Reader) -> bam::Writer {
    let header = bam::Header::from_template(infile.header());
    bam::Writer::from_stdout(&header, bam::Format::BAM).unwrap()
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

                let bam_summary = summary(&mut bam);

                if !args.min && !args.max && !args.mean && !args.reads {
                   println!("min: {}", bam_summary.min);
                   println!("max: {}", bam_summary.max);
                   println!("mean: {}", bam_summary.mean);
                   println!("reads: {}", bam_summary.reads);
                } else if args.min {
                    println!("{}", bam_summary.min);
                } else if args.max {
                    println!("{}", bam_summary.max);
                } else if args.mean {
                    println!("{}", bam_summary.mean);
                } else if args.reads {
                    println!("{}", bam_summary.reads);
                }

                return
            },
            Bamf::Hist(args) => {
                let mut bam = create_infile_bam_connection(&args.infile);
                hist(&mut bam, args.below)
            },
            Bamf::Split(args) => {
                println!("{:?}", args)

            }
        }
    }


}

