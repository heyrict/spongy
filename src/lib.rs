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
pub struct Item<'a> {
    pub wrapper: Wrapper,
    pub text: &'a str,
}

/// The formatter
pub struct Formatter<'a> {
    text: &'a str,
}

/// Plain text or wrapped text
#[derive(Debug, PartialEq)]
pub enum Element<'a> {
    Text(&'a str),
    Wrapped(Item<'a>),
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
        Formatter { text }
    }

    /// Parse with a mapper function
    pub fn parse_with(&self, mapper: impl Fn(&Item) -> Option<String>) -> String {
        self.into_elements()
            .iter()
            .map(|el: &Element| -> String {
                match el {
                    Element::Text(t) => t.to_owned().to_owned(),
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
            return vec![Element::Text(self.text)];
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
                            self.text.get(prev_index..start_index).unwrap(),
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
                        text: self.text.get(prev_index..start_index).unwrap(),
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
                    elements.push(Element::Text(context.as_ref().unwrap().get_prefix()))
                }
                elements
            }
            1 => {
                // Last window, size of 1
                let end_index = start_index + 1;
                if context.is_none() {
                    if prev_index < end_index {
                        elements.push(Element::Text(self.text.get(prev_index..).unwrap()));
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
                            text: self.text.get(prev_index..start_index).unwrap(),
                        }));
                    } else {
                        elements.push(Element::Text(
                            self.text.get(prev_index - prefix.len()..).unwrap(),
                        ));
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
                Element::Text("Hello, "),
                Element::Wrapped(Item {
                    wrapper: Wrapper::Curly,
                    text: "name"
                }),
                Element::Text("!")
            ]
        );
    }

    #[test]
    fn parse_double_curly() {
        let formatter = Formatter::new("Hello, {{ user.name }}");
        assert_eq!(
            formatter.into_elements(),
            vec![
                Element::Text("Hello, "),
                Element::Wrapped(Item {
                    wrapper: Wrapper::DoubleCurly,
                    text: " user.name "
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
                Element::Text("A "),
                Element::Wrapped(Item {
                    wrapper: Wrapper::DollarCurly,
                    text: "plus",
                }),
                Element::Text(" B"),
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
                text: "",
            })]
        );

        let formatter = Formatter::new("{#}");
        assert_eq!(formatter.into_elements(), vec![Element::Text("{#}"),]);

        let formatter = Formatter::new("{# comment #}");
        assert_eq!(
            formatter.into_elements(),
            vec![Element::Wrapped(Item {
                wrapper: Wrapper::CurlyHash,
                text: " comment ",
            })]
        );
    }

    #[test]
    fn parse_curly_percent() {
        let formatter = Formatter::new("{%}");
        assert_eq!(formatter.into_elements(), vec![Element::Text("{%}"),]);

        let formatter = Formatter::new("Hello, {% name %}");
        assert_eq!(
            formatter.into_elements(),
            vec![
                Element::Text("Hello, "),
                Element::Wrapped(Item {
                    wrapper: Wrapper::CurlyPercent,
                    text: " name "
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
                text: "n"
            })]
        );

        let formatter = Formatter::new("{}");
        assert_eq!(
            formatter.into_elements(),
            vec![Element::Wrapped(Item {
                wrapper: Wrapper::Curly,
                text: "",
            })]
        );
    }

    #[test]
    fn parse_broken() {
        let formatter = Formatter::new("${");
        assert_eq!(formatter.into_elements(), vec![Element::Text("${")]);

        let formatter = Formatter::new("${todo..");
        assert_eq!(formatter.into_elements(), vec![Element::Text("${todo..")]);
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
