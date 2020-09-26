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
    /// a range of fragment sizes to split on: -s <min> <max>
    /// passing -s multiple times allows splitting into multiple ranges
    /// order of -s flag determines order of assignment to a range
    #[structopt(short = "s", long = "split", multiple = true, number_of_values = 2)]
    split: Vec<i64>,
    /// File prefix
    #[structopt(short = "o", long = "prefix", required = true)]
    prefix: String,
    /// Allow multimembership
    /// Whether to allow fragments to be assigned to more than one output file
    /// otherwise reads will be assigned to the first overlapping range based on
    /// the order indicated by -s
    #[structopt(short = "m", long = "multi")]
    multimembership: bool,

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

#[derive(Debug)]
struct FragmentRange {
    min: i64,
    max: i64
}

impl Copy for FragmentRange {}

impl Clone for FragmentRange {
    fn clone(&self) -> FragmentRange {
        FragmentRange{min: self.min, max: self.max}
    }
}

impl FragmentRange {
    fn new() -> FragmentRange {
        FragmentRange{min: 0, max: 1000}
    }
}

impl FragmentRange {
    fn suffix(&self) -> String {
        format!("_{}to{}", self.min, self.max)
    }
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

fn prepare_split_ranges(split: &Vec<i64>) -> Vec<FragmentRange> {
    // Prepare fragment ranges for each split
    // (where `split` is an even-numbered vector such that every 2 entries define the beginning and end of a range)
    // sorting or any other reorganization operations on `split` are destructive and will produce incorrect behavior
    // by unwrapping the split array into min/max pairs
    // and packaging into vector of structs
    // returns vector of FragmentRanges

    // One range for every pair of entries in `split`
    let mut ranges = vec![FragmentRange::new(); split.len() / 2];

    // Slide through split by 2's to pack into ranges
    let mut split_i = 0;
    let mut range_i = 0;
    while split_i + 1 <= split.len() {

        // Using min/max ensures split ranges don't need to be sorted correctly
        // ie `-s 120 20` == `-s 20 120`
        let (min, max) = {
            (
                std::cmp::min(split[split_i], split[split_i+1]),
                std::cmp::max(split[split_i], split[split_i+1])
            )
        };

        ranges[range_i] = FragmentRange{min: min, max: max};

        split_i += 2;
        range_i += 1;
    }

    ranges
}

fn create_infile_bam_connection(path: &std::path::PathBuf) -> bam::Reader {
    // TODO: I think instead of unwrap() I should use match() to handle err?
    // maybe .expect()?
    // maybe `?`
    bam::Reader::from_path(path).unwrap()
}

fn create_stdout_bam_connection(infile: &bam::Reader) -> bam::Writer {
    let header = bam::Header::from_template(infile.header());
    // TODO: I think instead of unwrap() I should use match() to handle err?
    // maybe .expect()?
    // maybe `?`
    bam::Writer::from_stdout(&header, bam::Format::BAM).unwrap()
}

fn create_file_bam_connection(path: &std::path::PathBuf, header: &bam::Header, out_prefix: Option<String>, out_suffix: Option<String>) -> bam::Writer {

    let output_path = output_path_from_prefix(path, out_prefix, out_suffix);
    //let err_msg = format!("Cannot open path to output file: {:?}", output_path);
    // Build writer
    //TODO: modify this to match ext to return a generic writer?
    bam::Writer::from_path(output_path, &header, bam::Format::BAM).unwrap()

}

fn output_path_from_prefix(input_path: &std::path::PathBuf, prefix: Option<String>, suffix: Option<String>) -> std::path::PathBuf {
    let mut output_path = String::new();

    // If specified, use prefix instead of input file location
    if let Some(x) = prefix {
        output_path.push_str(&x);
    } 

    if let Some(x) = suffix {
        output_path.push_str(&x);
    }

    // TODO: Figure out how to use this to parse filetype?
    // If not, then you can drop "input_path" from this and child functions
    if let Some(x) = input_path.extension() {
        let mut dot = String::from(".");
        dot.push_str(x.to_str().unwrap());

        output_path.push_str(&dot);
    }

    std::path::Path::new(&output_path).to_path_buf()

}

fn main() {

    let cli = Cli::from_args();

    if let Some(subcommand) = cli.commands{
        match subcommand {
            Bamf::Filter(args) => {

                let mut bam = create_infile_bam_connection(&args.infile);
                let mut out = create_stdout_bam_connection(&bam);

                for r in bam.records() {
                    let record = r.unwrap();
                    filter(&args, &record, &mut out);
                }

            },
            Bamf::Summary(args) => {
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
                //TODO: remove this
                //println!("{:?}", args);

                // Collect vector of ranges
                let split_ranges = prepare_split_ranges(&args.split);

                //TODO: remove this
                //println!("{:?}", split_ranges);
                //println!("{:?}", split_ranges[1].suffix());
           

                // TODO: add bam_out entry to FragmentRange?
                // Maybe add FragmentSplit struct which has FragmentRange and bam_out members
                // Instantiate output file connections w/ optional prefix + <min>to<max>.bam suffix

                let mut bam = create_infile_bam_connection(&args.infile);
                let header = bam::Header::from_template(&bam.header());

                let mut outputs = Vec::new();

                let file_prefix = &args.prefix;

                for i in 0..split_ranges.len() {
                    let range = &split_ranges[i];
                    let suffix = Some(range.suffix());

                    outputs.push(create_file_bam_connection(&args.infile, &header, Some(file_prefix.to_string()), suffix));
                    //  used when prefix was Option<String>
                    //if let Some(ref prefix) = file_prefix {
                    //    outputs.push(create_file_bam_connection(&args.infile, &header, Some(prefix.to_string()), suffix));
                    //} else {
                    //    outputs.push(create_file_bam_connection(&args.infile, &header, None, suffix));
                    //}
                }

                // open bam file, write reads to files corresponding to each range

                for r in bam.records() {

                    let record = r.unwrap();

                    let insert = record.insert_size().abs();

                    for ro in split_ranges.iter().zip(&mut outputs) {
                        let (range, out) = ro;
                        if insert >= range.min && insert <= range.max {
                            out.write(&record)
                               .expect("Cannot write to output file");

                            if !args.multimembership {
                                // if reads can't be assigned to more than one file
                                // move to next read, else check if it fits the next file
                                break
                            }

                        }
                    }

                }

            }
        }
    }


}

