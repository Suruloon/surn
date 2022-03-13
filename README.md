<div align="center">
     <p>
          <img width="150" alt="Surn" src="https://i.imgur.com/OEPbt6V.png">
     </p>
     <p>
          <p>
            A powerful programming language enabling you to write in one syntax, many languages, single form.
            </p>
     </p>
</div>

## What is Surn?

Surn is a powerful [source-to-source compiler](https://en.wikipedia.org/wiki/Source-to-source_compiler) that allows anyone to write in one syntax, but recieve the benefits of using multiple languages by compiling to that language. Surn is also a [compiler](https://en.wikipedia.org/wiki/Compiler) that can generate binaries with GCC.

#### Supported Languages

| Language  | Surn Version         | Source     |
| --------- | -------------------- | ---------- |
| PHP 8.x.x | `v0.0.1-alpha.rc.1`  | [master]() |
| C         | `v0.0.1-beta.rfc.12` |            |

### How to support a language?

Surn uses a poly filling language known as "smtt", pronounced "smitt". SMTT stands for Surn Mapped Token Tree. The syntax of smtt is very unique in that it is extermely limited.



You can view the specifications of [SMTT by clicking here.](/docs/smtt/README.md)

Here's an example that compiles the following code to JS.

The **surn** code:

```rust
use std::fmt;

var hello: string = "Hello, {?}";
var world: string = "World!";

pub fn main() {
    print(hello.fmt(world));   
}
```