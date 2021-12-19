```js
Program {
    body: [
        Statement {
            kind: "VariableDeclaration",
            value: VariableDeclaration {
                name: "a",
                value: null,
                type: "number",
            }
        },
        Statement {
            kind: "VariableDeclaration",
            value: VariableDeclaration {
                name: "c",
                value: MemberExpression {
                    property: "b",
                    from: MemberExpression {
                        property: "a",
                        from: MemberExpression {
                            property: "z",
                            from: CallExpression {
                                name: "f",
                                arguments: [1, 3 , 8],
                                returns: "number"
                            }
                        }
                    }
                }
            }
        }
    ]
}
```
Will be generated from:
```js
var a;
var c = b.a.z(f(1, 3, 8));
```

Syntax to support transpiling without using the api:
```yaml
# This is the name of the language Surn will recognize this config as.
language: php

# This is the name of the file that Surn will refer to when poly filling ast.
polyfill: resources/php-8.0.0.polyfill.yaml
```