# Zebra - JS Fuzzer

This is almost entirely based on fuzzilli and has all the similar concepts as
fuzzilli. The differences would be in the typing system of the IR and the
representation of inbuilt and runtime functions/objects.

To try it out, install rust
[https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install)
and run the fuzzer with the following - 

```sh
$ cargo run -- --file <path/to/js/engine> --dry-run -d
```

This will perform one round of fuzzing - generate a program, print it out to
stdout and run it with the engine. Try `cargo run -- --help` for all the
options.

The fuzzer still does not support coverage and mutation though.
