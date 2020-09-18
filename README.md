# BAMF 🦀
## A mediocre tool for manipulating bam entries by fragment size 

Each subcommand takes a bam file as input, streams output to stdout.

## SubCommands:
`filter` = filter bam file to keep only fragments of given size
 - `-a` = return all fragments equal to or above this fragment size
 - `-b` = return all fragments equal to or below this fragment size
 
`summary` = print summary statistics min/max/mean fragment size of the bam file

