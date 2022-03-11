# SMTT
Surn Mapped Token Tree

### What is SMTT?
SMTT is a language used to translate **surn** code to any language of your desire!

### Using SMTT to convert code to javascript
```rust
@name: Javascript
@file_type: .js .javascript

// We need to provide SMTT with the context of javascript!
// This is a little hard, to understand at first, but you will get used to it when you understand it.

// Let's support `let` and const bindings!
// First, we need to tell surn that there's a keyword associated with a "mutable variable"
label AssignMut = "let"
// Now we need to tell it that there's a word associated with a "constant variable"
label AssignConst = "const"

// Because javascript doesn't have any "types" we need to clean up types!
// We can do this by using the `@>` (plug) operation, however because this is more complex than just remapping
// We will need to provide the `<@` (unplug) operation as well.

@> AssignMut as x {
    // We're in a plug block.
    // Statements here are immediately translated.
    // `x` is an instance of `AssignMut`
    // Each time an `AssignMut` is encountered, we need to "plug" an empty value into the types.
    // We can do this by creating another plug!
    @> x.ty as types {
        // x.ty is an instance of `VarType`
        // However because we don't care about this, we can simple nop it out.
        return ""
    }
}

// Now that we "nop" out the types on `AssignMut` we need to do the same with `AssignConst`!
// because `AssignMut` and `AssignConst` are realitively the same, we can just use the `=>` operation
// which is used to copy an existing implementation to another implementation.
// The newly implementing implementation takes priority over the existing implementation in this operation.

impl AssignMut to! AssignConst

// Now that we have our plugging, let's do some unplugging (parsing)

if label == let {
    // we can unplug the current token stream for this label!
}
```