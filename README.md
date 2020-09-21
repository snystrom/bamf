# BAMF 🦀
## A mediocre tool for manipulating bam entries by fragment size 

### Install

[Install Rust](https://www.rust-lang.org/tools/install). It's super easy!

Install the binary
``` sh
cargo install --git https://github.com/snystrom/bamf
```

Test install
``` sh
bamf --version
```

## Usage
Each subcommand takes a bam file as input, streams output to stdout.

## SubCommands:
`filter` filter bam file to keep only fragments of given size
 - `-a (--above)` return all fragments equal to or above this fragment size
 - `-b (--below)` return all fragments equal to or below this fragment size

`stats` print summary statistics min/max/mean fragment size and read count of the bam file

Flags passed to `stats` cause it to print only the value.
 - `-n (--min)` Print minimum fragment size
 - `-x (--max)` Print maximum fragment size
 - `-d (--mean)` Print mean fragment size
 - `-c (--reads)` Print total read count

`histogram` prints count of each fragment size in csv format
 - `-b (--below)` count all fragments equal to or below this size

Examples:
 ```
 # return all fragments >= 100 bp
 bamf filter -a 100 input.bam > output.bam
 
 # return all fragments <= 100 bp
 bamf filter -b 100 input.bam > output.bam
 
 # return all fragments between 150 and 700 bp
 bamf filter -a 150 -b 700 input.bam > output.bam

# Pretty print statistics to terminal
 bamf stats input.bam 
> min: 20
> max: 700
> mean: 300
> reads: 5000
 
# Print mean fragment size to terminal
 bamf stats --mean input.bam 
> 300

# Counts each fragment size in input.bam
 bamf histogram input.bam > input_histogram.csv
 ```

