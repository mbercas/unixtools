# unixtools

Toy project for Rust learning. Implementing simplified versions of the unix tools. Not necessarity 100% compliant with https://pubs.opengroup.org/onlinepubs/9699919799/utilities/contents.html but I try not to deviate too much

The titles below show the implementation sequence, I have been refactoring the code as I learn more.

## cat

Cat command;

  Basic CLI application

- file management; basic functions, discover if path is valid, read file contents.
- command line parsing; using `clap` crate for parsing command line.
- process: exit

## grep

  Search in files using regular expression

- vectors: basic vector operations; pre-assign memory, push, append.
- io: read file contents into buffers for more efficient disk access
- regex; match file contents using regex crate
- stdin



## Refactoring 1

### Use WriteBuf

Use WriteBuf to optimize stdout performance.

### toolslib

- Extract common functionality into a library,
- refactor functions for unittesting
   + add testing to github actions cargo test --all
- use Cargo workspaces
- implement github action

## hexdump

- Implement iterator struct for formatting the output. Ideally I would have used a generator, but this is still a experimental feature in Rust.
- Use some advanced methods of the `char` type.
