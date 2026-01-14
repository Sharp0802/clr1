# `clr1`

`clr1` is canonical LR(1) parser generator, written with Rust.

## Grammar

### Overview

- Lexer file (`test.lex`):

```
IPv4  : ([0-9]{1,3} '.'){4} ;
Number: [0-9]+ | ('0x' [0-9a-fA-F]+) ;
Ident : [a-zA-Z_][a-zA-Z0-9_]* ;
WS    : [ \t\n\r\v\f]*
```

- Parser file (`test.parse`):

```
# Hello, this is comment!

Value : IPv4 
      | Number 
      ;

# some_name = 127.0.0.1; (name = some_name, value = 127.0.0.1)
Assign: Ident@name WS '=' WS Value@value WS ';' ;
```

> [!NOTE]
> Lexer file format and parser file format are same

### Rule

```
<rule name> = <pattern> ;
```

- Rule name should be `[a-zA-Z0-9]+`

### Pattern

References another rule

- Literal(`''`)

```
'<literal>'
```

String literal.
Characters can be escaped as below:

|    Escape    | Desc.                                          |
|:------------:|:-----------------------------------------------|
|     `\'`     | `'`                                            |
|     `\\`     | `\`                                            |
|     `\t`     | Horizontal Tab (0x9)                           |
|     `\n`     | Line Feed (0xA)                                |
|     `\v`     | Vertical Tab (0xB)                             |
|     `\f`     | Form Feed (0xC)                                |
|     `\r`     | Carriage Return (0xD)                          |
|     `\(`     | `(`                                            |
|     `\)`     | `)`                                            |
|     `\{`     | `{`                                            |
|     `\}`     | `}`                                            |
|     `\[`     | `[`                                            |
|     `\]`     | `]`                                            |
|    `\xXX`    | 1-byte Unicode (`0xXX`)                        |
| `\u{UUU...}` | Variable-length Unicode (same as `\u` in Rust) |

- Class(`[]`)

```
[<allow-list>]
[^<deny-list>]
```

Character class.
Escape sequence described in string literal section can be used.

- Quantifier(`?`, `*`, `+`, `{}`)

```
<pattern>?         (1)
<pattern>*         (2)
<pattern>+         (3)
<pattern>{min,}    (4)
<pattern>{,max}    (5)
<pattern>{min,max} (6)
<pattern>{exact}   (7)
```

1. Optional (0..1)
2. Zero-or-More (0..inf)
3. One-or-More (1..inf)
4. At least (min..inf)
5. At most (0..max)
6. Range (min..max)
7. Exact (exact..exact)

- Group(`()`)

```
(<pattern>...)
```

Sequence of patterns.

- Or(`|`)

```
<pattern a>|<pattern b>
```

Match `<pattern a>` or `<pattern b>`.

> [!WARN]
> Lexer will try them deterministically (ordered),
> while parser will try them non-deterministically (unordered).

- Named

```
<pattern>@<name>
```

Name such `<pattern>` as `<name>`.
Object that captured with named pattern will be populated to be used after parsing phase. 

- Reference

```
<name>
```

Reference another rule named as `<name>`.
