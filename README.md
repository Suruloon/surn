<div align="center">
     <p>
          <img width="150" alt="Surn" src="https://i.imgur.com/OEPbt6V.png">
     </p>
     <p>
          <p>
            A powerful transpiler enabling you to write in one syntax, many languages, single form.
            </p>
     </p>
</div>

## What is Surn?

Surn is a powerful [source-to-source compiler](https://en.wikipedia.org/wiki/Source-to-source_compiler) that allows anyone to write in one syntax, but recieve the benefits of using multiple languages by compiling to that language.

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

The `js.sast` file:

```c
format

translate VarStatement(var @> name, assignment):
    name = var.name
    right = var.assignment?

    return:
        // Anything below a return block with proper indentation is translated
        let $name = $assignment;

translate FunctionStatement(fn @> name):
    args = fn.inputs map(input: input.name)

    return:
        function $name(${"," <@ args}) {
            ${NEW_LINE <@ fn.body}
        }
    }

translate CallExpression(call @> name, args) {
    return:
        {name}({", " <@ args});
}

support [std.fmt] fmt((owner is String), (append is Any)):
    return:
        function fmt(owner, append) {
            // js has it's own conversion for strings
            return $owner.split("{?}").join($append.toString());
        }

support [std.io] print((v is Any)):
    return:
        function print(v) {
            console.log(v);
        }
```

Translates to:

```js
/**
 * This file was gernerated by the Surn Compiler.
 * Surn: https://github.com/Suruloon/surn
 *
 * @std: std-js options: [mangled: true]
 *
 * Copyright (c) 2021, Suruloon Studios. All rights reserved.
 * Licensed under the CC0-1.0 License.
 */
function a1krm(_e, _d) {
    return _e.split('{?}').join(_d);
}

function b93af(_a) {
    console.log(_a);
}

/**
 * User script
 * @author surn
 */
let hello = "Hello, {?}";
let world = "World!";

function main() {
    b93af(a129(hello, world));
}
```
