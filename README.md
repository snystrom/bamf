# BAMF 🦀
## A mediocre tool for manipulating bam entries by fragment size 

### Install

Install rust if you haven't by visiting: [Install Rust](https://www.rust-lang.org/tools/install). It's super easy!

Clone this repository
``` sh
git clone <link>
```

Build using cargo
``` sh
cd bamf/
cargo build --release
```

Install the binary
``` sh
mv target/release/bamf ~/bin/

# Optional if ~/bin/ is already in your path
echo "export PATH=\$PATH:~/bin/" >> ~/.bashrc
```

Test install

``` sh
bamf --version
```

## Usage
Each subcommand takes a bam file as input, streams output to stdout.

## SubCommands:
`filter` filter bam file to keep only fragments of given size
 - `-a` return all fragments equal to or above this fragment size
 - `-b` return all fragments equal to or below this fragment size

Examples:
 ```
 # return all fragments >= 100 bp
 bamf filter -a 100 input.bam > output.bam
 
 # return all fragments <= 100 bp
 bamf filter -b 100 input.bam > output.bam
 
 # return all fragments between 150 and 700 bp
 bamf filter -a 150 -b 700 input.bam > output.bam
 ```
 
`summary` print summary statistics min/max/mean fragment size of the bam file

Examples:
```
bamf summary input.bam
```

