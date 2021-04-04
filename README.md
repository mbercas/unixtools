# unixtools

Toy project for Rust learning. Implementing simplified versions of the unix tools. Not necessarity 100% compliant with https://pubs.opengroup.org/onlinepubs/9699919799/utilities/contents.html but I try not to deviate too much

## cat

Cat command; 

- file management; basic functions, discover if path is valid, read file contents.
- command line parsing; using `clap` crate for parsing command line.
- process: exit 

## grep

- vectors: basic vector operations; pre-assign memory, push, append.
- io: read file contents into buffers for more efficient disk access
- regex; match file contents using regex crate
- stdin
