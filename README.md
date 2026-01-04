# KCSdecoder
Tool for decoding data from audio files using Kansas City Standard.

I made this tool because I needed to dump some data casette tapes for a Nascom system, which is using the Kansas City Standard format. It should also be usable with some other systems, however I only ever tested it with Nascom files.
It is very much a prototype tool designed for my own specific needs and it was one of the first programs I ever wrote using Rust, so the code may not be very pretty.

## Building
Prerequisites: Rust toolchain, see [link](https://rust-lang.github.io/rustup/installation/index.html).

Build the tool by typing `cargo build`.

The tool should build and can be run like this (on Linux):
`./target/debug/kcs_decoder --help`

## Using
Input files are WAVE files and since there is no internal upsampling at the moment, it is beneficial to upsample all recordings to e.g. 192kHz before processing.

To decode a recording containing Nascom software, I used it like this:
`./target/debug/kcs_decoder --preset NASCOM recording.wav`

If data could be decoded, it will write a number of .dat files containing this data with the time stamp of where the data was found.

To validate the data from NASCOM tapes, there is another tool `NASCOM/nascom_cas_verify.py` to help with that.

This Python script will check a .dat file for the pilot tone, headers, checksums etc. and report if all blocks were correct and accounted for. If everything was accounted for, it will write a cleaned file.
It something was missing, it will not automatically write the output file, however if a filename is given as the second parameter, it will write the good blocks it had found so it can be manually recovered.
