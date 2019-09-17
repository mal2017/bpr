# bpr

`bpr` is for quickly generating a few pseudoreplicate bam files for operations
like `idr`.

Keep in mind that I whipped this up very quickly for a specific case, so I can't guarantee it isn't filled with bugs.

## usage

You only need to provide a bam file (cram and sam soon to come), an output base file name, and a string for a random seed.

```bash
bpr a.bam a_pseudo --seed hello
```

## see also

https://github.com/nboley/idr
