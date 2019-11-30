type Middleware = Box<dyn Fn(&Item) -> Option<&str>>;

/// Wrappers over the text
#[derive(Clone, Debug, PartialEq)]
pub enum Wrapper {
    DoubleCurly,
    Curly,
    DollarCurly,
    CurlyHash,
    CurlyPercent,
}

/// Wrapped text
#[derive(Debug, PartialEq)]
pub struct Item {
    pub wrapper: Wrapper,
    pub text: String,
}

/// The formatter
pub struct Formatter<'a> {
    text: &'a str,
    middlewares: Vec<Middleware>,
}

/// Plain text or wrapped text
#[derive(Debug, PartialEq)]
pub enum Element {
    Text(String),
    Wrapped(Item),
}

impl Wrapper {
    /// get the prefix of the wrapper
    fn get_prefix(&self) -> &'static str {
        match self {
            Wrapper::DoubleCurly => "{{",
            Wrapper::Curly => "{",
            Wrapper::DollarCurly => "${",
            Wrapper::CurlyPercent => "{%",
            Wrapper::CurlyHash => "{#",
        }
    }

    /// get the suffix of the wrapper
    fn get_suffix(&self) -> &'static str {
        match self {
            Wrapper::DoubleCurly => "}}",
            Wrapper::Curly | Wrapper::DollarCurly => "}",
            Wrapper::CurlyPercent => "%}",
            Wrapper::CurlyHash => "#}",
        }
    }

    /// Get all wrappers
    fn values() -> [Self; 5] {
        // Order count to avoid conflicts.
        // Wrappers with more characters should have a higher priority.
        [
            Wrapper::DoubleCurly,
            Wrapper::CurlyPercent,
            Wrapper::CurlyHash,
            Wrapper::DollarCurly,
            Wrapper::Curly,
        ]
    }
}

impl<'a> Formatter<'a> {
    /// Initialize a new formatter
    pub fn new(text: &'a str) -> Formatter<'a> {
        Formatter {
            text,
            middlewares: vec![],
        }
    }

    /// Adds a middleware to the formatter
    pub fn add_middleware(mut self, middleware: Middleware) -> Self {
        self.middlewares.push(middleware);
        self
    }

    /// Parse text with given middlewares
    pub fn parse(&self) -> String {
        self.into_elements()
            .iter()
            .map(|el: &Element| -> String {
                match el {
                    Element::Text(t) => t.to_owned(),
                    Element::Wrapped(item) => {
                        for middleware in &self.middlewares {
                            let processed = middleware(&item);
                            if processed.is_some() {
                                return processed.unwrap().to_owned();
                            }
                        }

                        format!(
                            "{}{}{}",
                            item.wrapper.get_prefix(),
                            item.text,
                            item.wrapper.get_suffix()
                        )
                    }
                }
            })
            .collect::<Vec<String>>()
            .join("")
    }

    pub fn parse_with(&self, mapper: impl Fn(&Item) -> Option<String>) -> String {
        self.into_elements()
            .iter()
            .map(|el: &Element| -> String {
                match el {
                    Element::Text(t) => t.to_owned(),
                    Element::Wrapped(item) => mapper(item).unwrap_or(format!(
                        "{}{}{}",
                        item.wrapper.get_prefix(),
                        item.text,
                        item.wrapper.get_suffix()
                    )),
                }
            })
            .collect::<Vec<String>>()
            .join("")
    }

    /// Convert text into a sequence of elements
    pub fn into_elements(&self) -> Vec<Element> {
        if self.text.len() < 2 {
            return vec![Element::Text(self.text.to_owned())];
        }
        let mut context: Option<Wrapper> = None;
        let mut new_context: Option<Wrapper> = None;
        let mut elements: Vec<Element> = vec![];

        // Current window
        //
        // # Example
        // ```
        // {}, Hello, {world}
        //   |        | |
        //   |        | end_index
        //   |        start_index
        //   prev_index
        // ```
        let mut start_index = 0;
        let mut prev_index = 0;

        let num_chars = self.text.bytes().count();
        let chars_list: Vec<u8> = self.text.bytes().collect();

        for end_index in 0..num_chars + 1 {
            // Require the size of window to be 2
            if end_index - start_index < 2 {
                continue;
            }

            let window = chars_list.get(start_index..end_index).unwrap();

            // When it is not inside a wrapper currently
            if context.is_none() {
                // Check if current char(s) matches a prefix.
                // If so, update the new_context.
                for w in Wrapper::values().iter() {
                    let mut wrapper_chars = w.get_prefix().bytes();
                    let is_match = match wrapper_chars.len() {
                        1 => wrapper_chars.next().unwrap() == window[0],
                        2 => {
                            wrapper_chars.next().unwrap() == window[0]
                                && wrapper_chars.next().unwrap() == window[1]
                        }
                        _ => false,
                    };

                    if is_match {
                        new_context = Some(w.clone());
                        break;
                    }
                }

                // If a new wrapper is created, create a text element
                if let Some(wrapper) = &new_context {
                    if start_index - prev_index > 0 {
                        elements.push(Element::Text(
                            self.text.get(prev_index..start_index).unwrap().to_owned(),
                        ));
                    }
                    // Update current window
                    match wrapper.get_prefix().len() {
                        1 => {
                            start_index += 1;
                            prev_index = start_index;
                        }
                        2 => {
                            start_index += 2;
                            prev_index = start_index;
                        }
                        _ => {
                            panic!("Suffix length should be less than 3");
                        }
                    }
                // Update current texts
                } else {
                    start_index += 1;
                };
            // When it is inside a wrapper currently
            } else {
                // Check if current char(s) matches the suffix.
                let wrapper = context.clone().unwrap();
                let suffix_len = wrapper.get_suffix().len();
                let mut wrapper_chars = wrapper.get_suffix().bytes();
                let is_match = match suffix_len {
                    1 => wrapper_chars.next().unwrap() == window[0],
                    2 => {
                        wrapper_chars.next().unwrap() == window[0]
                            && wrapper_chars.next().unwrap() == window[1]
                    }
                    _ => false,
                };

                // If so, update the new_context.
                if is_match {
                    new_context = None;
                    elements.push(Element::Wrapped(Item {
                        wrapper: wrapper.clone(),
                        text: self.text.get(prev_index..start_index).unwrap().to_owned(),
                    }));

                    // Update current_texts
                    match suffix_len {
                        1 => {
                            prev_index = end_index - 1;
                            start_index = end_index - 1;
                        }
                        2 => {
                            prev_index = end_index;
                            start_index = end_index;
                        }
                        _ => {
                            panic!("Suffix length should be less than 3");
                        }
                    }
                } else {
                    start_index = end_index - 1;
                };
            }
            context = new_context.clone();
        }

        match num_chars - start_index {
            0 => {
                if context.is_some() {
                    elements.push(Element::Text(
                        context.as_ref().unwrap().get_prefix().to_owned(),
                    ))
                }
                elements
            }
            1 => {
                // Last window, size of 1
                let end_index = start_index + 1;
                if context.is_none() {
                    if prev_index < end_index {
                        elements.push(Element::Text(
                            self.text.get(prev_index..).unwrap().to_owned(),
                        ));
                    };
                } else {
                    let prefix = context.as_ref().unwrap().get_prefix();
                    let suffix = context.as_ref().unwrap().get_suffix();

                    // If end_index matches the closing wrapper
                    let is_match = match suffix.len() {
                        1 => {
                            &suffix.bytes().next().unwrap()
                                == chars_list.get(end_index - 1).unwrap()
                        }
                        _ => false,
                    };

                    if is_match {
                        elements.push(Element::Wrapped(Item {
                            wrapper: context.unwrap(),
                            text: self.text.get(prev_index..start_index).unwrap().to_owned(),
                        }));
                    } else {
                        let mut s = String::from(prefix);
                        s.push_str(self.text.get(prev_index..).unwrap());
                        elements.push(Element::Text(s));
                    }
                };
                elements
            }
            _ => panic!("Unreachable statement"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_curly() {
        let formatter = Formatter::new("Hello, {name}!");
        assert_eq!(
            formatter.into_elements(),
            vec![
                Element::Text("Hello, ".to_owned()),
                Element::Wrapped(Item {
                    wrapper: Wrapper::Curly,
                    text: "name".to_owned()
                }),
                Element::Text("!".to_owned())
            ]
        );
    }

    #[test]
    fn parse_double_curly() {
        let formatter = Formatter::new("Hello, {{ user.name }}");
        assert_eq!(
            formatter.into_elements(),
            vec![
                Element::Text("Hello, ".to_owned()),
                Element::Wrapped(Item {
                    wrapper: Wrapper::DoubleCurly,
                    text: " user.name ".to_owned()
                })
            ]
        );
    }

    #[test]
    fn parse_dollar_curly() {
        let formatter = Formatter::new("A ${plus} B");
        assert_eq!(
            formatter.into_elements(),
            vec![
                Element::Text("A ".to_owned()),
                Element::Wrapped(Item {
                    wrapper: Wrapper::DollarCurly,
                    text: "plus".to_owned(),
                }),
                Element::Text(" B".to_owned()),
            ]
        );
    }

    #[test]
    fn parse_curly_hash() {
        let formatter = Formatter::new("{##}");
        assert_eq!(
            formatter.into_elements(),
            vec![Element::Wrapped(Item {
                wrapper: Wrapper::CurlyHash,
                text: String::new(),
            })]
        );

        let formatter = Formatter::new("{#}");
        assert_eq!(
            formatter.into_elements(),
            vec![Element::Text("{#}".to_owned()),]
        );

        let formatter = Formatter::new("{# comment #}");
        assert_eq!(
            formatter.into_elements(),
            vec![Element::Wrapped(Item {
                wrapper: Wrapper::CurlyHash,
                text: " comment ".to_owned(),
            })]
        );
    }

    #[test]
    fn parse_curly_percent() {
        let formatter = Formatter::new("{%}");
        assert_eq!(
            formatter.into_elements(),
            vec![Element::Text("{%}".to_owned()),]
        );

        let formatter = Formatter::new("Hello, {% name %}");
        assert_eq!(
            formatter.into_elements(),
            vec![
                Element::Text("Hello, ".to_owned()),
                Element::Wrapped(Item {
                    wrapper: Wrapper::CurlyPercent,
                    text: " name ".to_owned()
                })
            ]
        );
    }

    #[test]
    fn parse_short() {
        let formatter = Formatter::new("{n}");
        assert_eq!(
            formatter.into_elements(),
            vec![Element::Wrapped(Item {
                wrapper: Wrapper::Curly,
                text: "n".to_owned()
            })]
        );

        let formatter = Formatter::new("{}");
        assert_eq!(
            formatter.into_elements(),
            vec![Element::Wrapped(Item {
                wrapper: Wrapper::Curly,
                text: String::new(),
            })]
        );
    }

    #[test]
    fn parse_broken() {
        let formatter = Formatter::new("${");
        assert_eq!(
            formatter.into_elements(),
            vec![Element::Text("${".to_owned())]
        );

        let formatter = Formatter::new("${todo..");
        assert_eq!(
            formatter.into_elements(),
            vec![Element::Text("${todo..".to_owned())]
        );
    }

    #[test]
    fn format_string() {
        let formatter = Formatter::new("{{greeting}}, {name}! by {hidden}")
            .add_middleware(Box::new(|item: &Item| -> Option<&str> {
                match item.wrapper {
                    Wrapper::Curly => match item.text.as_ref() {
                        "name" => Some("world"),
                        _ => None,
                    },
                    _ => None,
                }
            }))
            .add_middleware(Box::new(|item| match item.wrapper {
                Wrapper::DoubleCurly => match item.text.as_ref() {
                    "greeting" => Some("Hello"),
                    _ => None,
                },
                _ => None,
            }));
        assert_eq!(formatter.parse(), "Hello, world! by {hidden}");
    }

    #[test]
    fn format_string_with() {
        let formatter = Formatter::new("{{greeting}}, {name}! by {hidden}");
        let parsed = formatter.parse_with(|item: &Item| -> Option<String> {
            match item.wrapper {
                Wrapper::Curly => match item.text.as_ref() {
                    "name" => Some("world".to_owned()),
                    _ => None,
                },
                Wrapper::DoubleCurly => match item.text.as_ref() {
                    "greeting" => Some("Hello".to_owned()),
                    _ => None,
                },
                _ => None,
            }
        });
        assert_eq!(parsed, "Hello, world! by {hidden}");
    }
}
