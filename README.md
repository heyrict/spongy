# Spongy
A minimal runtime string formatter as a PoC for starship format configuration.

**NOTE: The API is not stable and may change any time**

## Why Spongy?
Most template engines or runtime formatters exists do not allow manipulation on the text inside the braces.
Often they use a fixed format `{:2d}`, or a struct property `{user.name}`.

As described in [starship/starship#624](https://github.com/starship/starship/issues/624), we would like some formatter that can parse something complex as `{segment?style=value}`,
so it would be great if we can get the text in the curly braces.

## The design
Fed with a string `Hello, {name}!`, the formatter should return

```
[
  Text ( "Hello, " ),
  Item (
    wrapper: Wrapper::Curly,
    text: "name",
  )
  Text ( "!" ),
]
```

## Usage
```rust
use spongy::{Formatter, Wrapper};

let formatted = parse_with("{{greeting}}, {name}!", |item| match item.wrapper {
    Wrapper::Curly => match item.text.as_ref() {
        "name" => Some("world".to_owned()),
        _ => None,
    },
    Wrapper::DoubleCurly => match item.text.as_ref() {
        "greeting" => Some("Hello".to_owned()),
        _ => None,
    },
    _ => None,
});
assert_eq!(formatted, "Hello, world!");
```
