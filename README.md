# Spongy
A minimal runtime string formatter as a PoC for starship format configuration.

## Usage
```rust
use spongy::Formatter;

let formatter =
    Formatter::new("Hello, {name}!").add_middleware(Box::new(|item| match item.wrapper {
        Wrapper::Curly => match item.text.as_ref() {
            "name" => Some("world"),
            _ => None,
        },
        _ => None,
    }));
assert_eq!(formatter.parse(), "Hello, world!");
```
