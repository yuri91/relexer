# Relexer

A simple and easy to use lexer for rust. Powered by custom derive

## Example

```

#[macro_use]
extern crate relexer;

#[derive(Debug, Token)]
pub enum Token {
    #[expr="\\*"]
    STAR,
    #[expr="\\+"]
    PLUS,
    #[expr="([01])"]
    VALUE(i32),
    #[expr="([a-z])([0-9])"]
    ID(String,i32),
    #[expr="[\t\n\r ]"]
    #[skip]
    WHITESPACE,
}

use std::fs::File;
use std::io::Read;
fn main() {

    let mut f = File::open("examples/test.txt").expect("input file not found");
    let mut input = String::new();
    f.read_to_string(&mut input).expect("input file not valid utf8");
    for t in relexer::scan::<Token>(&input) {
        println!("{:?}", t)
    }
}
```
