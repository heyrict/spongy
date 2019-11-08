type Middleware = Box<dyn Fn(&Item) -> Option<String>>;

#[derive(Clone, Debug, PartialEq)]
pub enum Wrapper {
    DoubleCurly,
    Curly,
    DollarCurly,
    CurlyHash,
    CurlyPercent,
}

#[derive(Debug, PartialEq)]
pub struct Item {
    pub wrapper: Wrapper,
    pub text: String,
}

pub struct Formatter<'a> {
    text: &'a str,
    middlewares: Vec<Middleware>,
}

impl<'a> std::fmt::Debug for Formatter<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        f.write_fmt(format_args!("Fomatter {{ text = \"{}\" }}", self.text))
    }
}

#[derive(Debug, PartialEq)]
pub enum Element {
    Text(String),
    Wrapped(Item),
}

impl Wrapper {
    fn get_prefix(&self) -> &'static str {
        match self {
            Wrapper::DoubleCurly => "{{",
            Wrapper::Curly => "{",
            Wrapper::DollarCurly => "${",
            Wrapper::CurlyPercent => "{%",
            Wrapper::CurlyHash => "{#",
        }
    }

    fn get_suffix(&self) -> &'static str {
        match self {
            Wrapper::DoubleCurly => "}}",
            Wrapper::Curly | Wrapper::DollarCurly => "}",
            Wrapper::CurlyPercent => "%}",
            Wrapper::CurlyHash => "#}",
        }
    }

    fn values() -> [Self; 5] {
        [
            // Two chars
            Wrapper::DoubleCurly,
            Wrapper::DollarCurly,
            Wrapper::CurlyPercent,
            Wrapper::CurlyHash,
            // One char
            Wrapper::Curly,
        ]
    }
}

impl<'a> Formatter<'a> {
    pub fn new(text: &'a str) -> Formatter<'a> {
        Formatter {
            text,
            middlewares: vec![],
        }
    }

    pub fn add_middleware(mut self, middleware: Middleware) -> Self {
        self.middlewares.push(middleware);
        self
    }

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
                                return processed.unwrap();
                            }
                        }
                        item.text.to_owned()
                    }
                }
            })
            .collect::<Vec<String>>()
            .join("")
    }

    fn into_elements(&self) -> Vec<Element> {
        if self.text.len() < 2 {
            return vec![Element::Text(self.text.to_owned())];
        }
        // TODO Implement this
        let mut prev_char: char = ' ';
        let mut context: Option<Wrapper> = None;
        let mut new_context: Option<Wrapper> = None;
        let mut elements: Vec<Element> = vec![];
        let mut current_texts = String::new();

        for c in self.text.chars() {
            // When it is not inside a wrapper currently
            if context.is_none() {
                // Check if current char(s) matches a prefix.
                // If so, update the new_context.
                for w in Wrapper::values().into_iter() {
                    let mut wrapper_chars = w.get_prefix().chars();
                    let is_match = match w.get_prefix().len() {
                        1 => wrapper_chars.nth(0).unwrap() == c,
                        2 => {
                            let first = wrapper_chars.next();
                            let second = wrapper_chars.next();
                            first == Some(prev_char) && second == Some(c)
                        }
                        _ => false,
                    };

                    if is_match {
                        new_context = Some(w.clone());
                        break;
                    }
                }

                if new_context.is_some() && current_texts.len() > 0 {
                    elements.push(Element::Text(current_texts.clone()));
                    current_texts = String::new();
                } else {
                    // Add char to text buffer;
                    current_texts.push(prev_char);
                }
            } else {
                let wrapper = context.clone().unwrap();
                let prefix_len = wrapper.get_prefix().len();
                let suffix_len = wrapper.get_suffix().len();
                let mut wrapper_chars = wrapper.get_suffix().chars();
                let is_match = match wrapper.get_suffix().len() {
                    1 => wrapper_chars.nth(0).unwrap() == c,
                    2 => {
                        let first = wrapper_chars.next();
                        let second = wrapper_chars.next();
                        first == Some(prev_char) && second == Some(c)
                    }
                    _ => false,
                };

                if is_match {
                    current_texts.push(c);
                    new_context = None;
                    let text_len = current_texts.len();
                    dbg!(&text_len, &current_texts, &suffix_len);
                    elements.push(Element::Wrapped(Item {
                        wrapper: wrapper.clone(),
                        text: current_texts
                            .get(prefix_len..(text_len - suffix_len))
                            .unwrap()
                            .to_owned(),
                    }));
                    current_texts = String::new();
                }
            }

            context = new_context.clone();
            prev_char = c;
        }

        elements
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_general() {
        let formatter = Formatter::new("Hello, {name}");
        assert_eq!(
            formatter.into_elements(),
            vec![
                Element::Text("Hello, ".to_owned()),
                Element::Wrapped(Item {
                    wrapper: Wrapper::Curly,
                    text: "name".to_owned()
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
    }

    #[test]
    fn parse_empty() {
        let formatter = Formatter::new("A ${} B");
        assert_eq!(
            formatter.into_elements(),
            vec![
                Element::Text("A ".to_owned()),
                Element::Wrapped(Item {
                    wrapper: Wrapper::DollarCurly,
                    text: String::new()
                }),
                Element::Text(" B".to_owned()),
            ]
        );
    }
}
