// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::stream;
use crate::stream::Stream;
use crate::error::Error;

/// CSS combinator.
#[derive(PartialEq,Debug)]
pub enum Combinator {
    /// Descendant selector
    Space,
    /// Child selector
    GreaterThan,
    /// Adjacent sibling selector
    Plus,
    /// General sibling selector
    Tilde,
}

/// CSS token.
#[derive(PartialEq,Debug)]
pub enum Token<'a> {
    /// Universal selector
    ///
    /// https://www.w3.org/TR/CSS21/selector.html#universal-selector
    UniversalSelector,
    /// Type selector
    ///
    /// https://www.w3.org/TR/CSS21/selector.html#type-selectors
    TypeSelector(&'a str),
    /// ID selector
    ///
    /// Value contains ident without `#`.
    ///
    /// https://www.w3.org/TR/CSS21/selector.html#id-selectors
    IdSelector(&'a str),
    /// Class selector
    ///
    /// Value contains ident without `.`.
    ///
    /// https://www.w3.org/TR/CSS21/selector.html#class-html
    ClassSelector(&'a str),
    /// Attribute selector
    ///
    /// We do not parse it's content yet, so value contains everything between `[]`.
    ///
    /// https://www.w3.org/TR/CSS21/selector.html#attribute-selectors
    AttributeSelector(&'a str),
    /// Pseudo-class
    ///
    /// Value contains ident without `:`.
    /// Selector: `"nth-child"`, value: The thing between the braces - `Some("3")`
    ///
    /// https://www.w3.org/TR/CSS21/selector.html#pseudo-class-selectors
    PseudoClass {
        /// The selector name (e.g., "nth-child", "hover")
        selector: &'a str,
        /// The optional value inside parentheses (e.g., "3" in ":nth-child(3)")
        value: Option<&'a str>,
    },
    /// `Combinator`
    Combinator(Combinator),
    /// Rules separator
    ///
    /// https://www.w3.org/TR/CSS21/selector.html#grouping
    Comma,
    /// Block start
    ///
    /// Indicates `{`.
    ///
    /// https://www.w3.org/TR/CSS21/syndata.html#rule-sets
    BlockStart,
    /// Block end
    ///
    /// Indicates `}`.
    ///
    /// https://www.w3.org/TR/CSS21/syndata.html#rule-sets
    BlockEnd,
    /// Declaration
    ///
    /// Contains property name and property value.
    ///
    /// https://www.w3.org/TR/CSS21/syndata.html#declaration
    Declaration(&'a str, &'a str),
    /// `@` rule (excluding the `@` sign itself). The content is not parsed,
    /// for example `@keyframes mymove` = `AtRule("keyframes"), AtStr("mymove")`.
    AtRule(&'a str),
    /// Raw Str inside of block
    DeclarationStr(&'a str),
    /// String following an @rule
    AtStr(&'a str),
    /// Same as PseudoClass, but with two colons (`::thing`).
    DoublePseudoClass {
        /// The selector name (e.g., "before", "after")
        selector: &'a str,
        /// The optional value inside parentheses
        value: Option<&'a str>,
    },
    /// End of stream
    ///
    /// Parsing is finished.
    EndOfStream,
}

#[derive(PartialEq)]
enum State {
    Rule,
    Declaration,
    DeclarationRule,
}

/// CSS tokenizer.
pub struct Tokenizer<'a> {
    stream: Stream<'a>,
    state: State,
    after_selector: bool,
    has_at_rule: bool,
    at_start: bool,
    /// Track nesting depth for nested @-rules support
    /// Each entry is true if the block at that level was started by an @-rule
    nesting_stack: Vec<bool>,
}

impl<'a> Tokenizer<'a> {
    /// Constructs a new `Tokenizer`.
    pub fn new(text: &str) -> Tokenizer<'_> {
        Tokenizer {
            stream: Stream::new(text.as_bytes()),
            state: State::Rule,
            after_selector: false,
            has_at_rule: false,
            at_start: true,
            nesting_stack: Vec::new(),
        }
    }

    /// Constructs a new bounded `Tokenizer`.
    ///
    /// It can be useful if CSS data is inside other data, like HTML.
    /// Using this method you will get an absolute error positions and not relative,
    /// like when using [`new()`].
    ///
    /// [`new()`]: #method.new
    pub fn new_bound(text: &str, start: usize, end: usize) -> Tokenizer<'_> {
        Tokenizer {
            stream: Stream::new_bound(text.as_bytes(), start, end),
            state: State::Rule,
            after_selector: false,
            has_at_rule: false,
            at_start: true,
            nesting_stack: Vec::new(),
        }
    }

    /// Returns a current position in the text.
    pub fn pos(&self) -> usize {
        self.stream.pos()
    }

    /// Parses a next token.
    pub fn parse_next(&mut self) -> Result<Token<'a>, Error> {
        if self.at_start {
            self.stream.skip_spaces();
            self.at_start = false;
        }

        if self.stream.at_end() {
            return Ok(Token::EndOfStream);
        }

        match self.state {
            State::Rule         => self.consume_rule(),
            State::Declaration  => self.consume_declaration(),
            State::DeclarationRule => self.consume_declaration(),
        }
    }

    fn consume_rule(&mut self) -> Result<Token<'a>, Error> {
        match self.stream.curr_char_raw() {
            b'@' => {
                self.after_selector = true;
                self.has_at_rule = true;
                self.stream.advance_raw(1);
                let s = self.consume_ident()?;
                
                // Don't consume parentheses here - let the next parse_next() call handle it
                // Just return the @rule name
                return Ok(Token::AtRule(s));
            }
            b'#' => {
                self.after_selector = true;
                self.has_at_rule = false;
                self.stream.advance_raw(1);
                let s = self.consume_ident()?;
                return Ok(Token::IdSelector(s));
            }
            b'.' => {
                self.after_selector = true;
                self.has_at_rule = false;
                self.stream.advance_raw(1);
                let s = self.consume_ident()?;
                return Ok(Token::ClassSelector(s));
            }
            b'*' => {
                self.after_selector = true;
                self.has_at_rule = false;
                self.stream.advance_raw(1);
                self.stream.skip_spaces();
                return Ok(Token::UniversalSelector);
            }
            b':' => {
                self.after_selector = true;
                self.has_at_rule = false;
                self.stream.advance_raw(1);

                // Whether this selector is a ::selector.
                let is_double_colon = self.stream.is_char_eq(b':')?;
                if is_double_colon {
                    self.stream.advance_raw(1); // consume the second :
                }

                let s = self.consume_ident()?;

                if self.stream.curr_char() == Ok(b'(') {
                    // Item is a thing()
                    self.stream.advance_raw(1); // (
                    let inner_len = self.stream.length_to(b')')?;
                    let inner = self.stream.read_raw_str(inner_len);
                    self.stream.advance_raw(1); // )
                    return Ok(if is_double_colon {
                        Token::DoublePseudoClass { selector: s, value: Some(inner) }
                    } else {
                        Token::PseudoClass { selector: s, value: Some(inner) }
                    });
                } else {
                    return Ok(if is_double_colon {
                        Token::DoublePseudoClass { selector: s, value: None }
                    } else {
                        Token::PseudoClass { selector: s, value: None }
                    });
                }
            }
            b'[' => {
                self.after_selector = true;
                self.has_at_rule = false;
                self.stream.advance_raw(1);
                let len = self.stream.length_to(b']')?;
                let s = self.stream.read_raw_str(len);
                self.stream.advance_raw(1); // ]
                self.stream.skip_spaces();
                return Ok(Token::AttributeSelector(s));
            }
            b',' => {
                self.after_selector = false;
                self.has_at_rule = false;
                self.stream.advance_raw(1);
                self.stream.skip_spaces();
                return Ok(Token::Comma);
            }
            b'{' => {
                // Track if this block was started by an @-rule
                self.nesting_stack.push(self.has_at_rule);
                self.after_selector = false;
                self.has_at_rule = false;
                self.state = State::Declaration;
                self.stream.advance_raw(1);
                return Ok(Token::BlockStart);
            }
            b'>' => {
                if self.after_selector {
                    self.after_selector = false;
                    self.has_at_rule = false;
                    self.stream.advance_raw(1);
                    self.stream.skip_spaces();
                    return Ok(Token::Combinator(Combinator::GreaterThan));
                } else {
                    return Err(Error::UnknownToken(self.stream.gen_error_pos()));
                }
            }
            b'+' => {
                if self.after_selector {
                    self.after_selector = false;
                    self.has_at_rule = false;
                    self.stream.advance_raw(1);
                    self.stream.skip_spaces();
                    return Ok(Token::Combinator(Combinator::Plus));
                } else {
                    return Err(Error::UnknownToken(self.stream.gen_error_pos()));
                }
            }
            b'~' => {
                if self.after_selector {
                    self.after_selector = false;
                    self.has_at_rule = false;
                    self.stream.advance_raw(1);
                    self.stream.skip_spaces();
                    return Ok(Token::Combinator(Combinator::Tilde));
                } else {
                    return Err(Error::UnknownToken(self.stream.gen_error_pos()));
                }
            }
            b'/' => {
                if self.consume_comment()? {
                    return self.parse_next();
                } else {
                    return Err(Error::UnknownToken(self.stream.gen_error_pos()));
                }
            }
            b'(' if self.has_at_rule => {
                // Parenthesized content in @-rule like @media (min-width: 800px)
                let s = self.consume_parenthesized_content()?;
                self.after_selector = true;
                return Ok(Token::AtStr(s));
            }
            _ => {
                if self.stream.is_space_raw() {
                    self.stream.skip_spaces();

                    if !self.after_selector {
                        return self.parse_next();
                    }

                    match self.stream.curr_char()? {
                        b'{' | b'/' | b'>' | b'+' | b'~' | b'*' | b'(' => { return self.parse_next(); },
                        _ => {
                            self.after_selector = false;
                            if !self.has_at_rule {
                                return Ok(Token::Combinator(Combinator::Space));
                            }
                        }
                    }
                }

                let s = self.consume_ident()?;
                let token_type = if self.has_at_rule {
                    self.has_at_rule = true;
                    Token::AtStr(s)
                } else {
                    self.has_at_rule = false;
                    Token::TypeSelector(s)
                };

                self.after_selector = true;
                return Ok(token_type);
            }
        }
    }

    fn consume_declaration(&mut self) -> Result<Token<'a>, Error> {
        self.stream.skip_spaces();

        match self.stream.curr_char_raw() {
            b'}' => {
                // Pop nesting level
                self.nesting_stack.pop();
                
                if self.state == State::DeclarationRule {
                    self.state = State::Declaration;
                } else if self.state == State::Declaration {
                    // Check if we should go back to Declaration or Rule based on nesting
                    if self.nesting_stack.is_empty() {
                        self.state = State::Rule;
                    } else {
                        // Stay in declaration mode for nested @-rules
                        self.state = State::Declaration;
                    }
                }
                self.has_at_rule = false;
                self.stream.advance_raw(1);
                self.stream.skip_spaces();
                return Ok(Token::BlockEnd);
            },
            b'{' => {
                // Track if this block was started by an @-rule
                self.nesting_stack.push(self.has_at_rule);
                self.has_at_rule = false;
                
                if self.state == State::Rule {
                    self.state = State::Declaration;
                } else if self.state == State::Declaration {
                    self.state = State::DeclarationRule;
                }
                self.stream.advance_raw(1);
                self.stream.skip_spaces();
                return Ok(Token::BlockStart);
            },
            b'@' => {
                // Nested @-rule inside a block (e.g., @media inside @os, or @os inside .class)
                self.after_selector = true;
                self.has_at_rule = true;
                self.stream.advance_raw(1);
                let s = self.consume_ident()?;
                self.stream.skip_spaces();
                return Ok(Token::AtRule(s));
            },
            b':' => {
                // Nested pseudo-class selector (e.g., :hover { } inside .button { })
                self.after_selector = true;
                self.has_at_rule = false;
                self.stream.advance_raw(1);

                // Check for ::pseudo-element
                let is_double_colon = self.stream.is_char_eq(b':')?;
                if is_double_colon {
                    self.stream.advance_raw(1);
                }

                let s = self.consume_ident()?;

                if self.stream.curr_char() == Ok(b'(') {
                    self.stream.advance_raw(1); // (
                    let inner_len = self.stream.length_to(b')')?;
                    let inner = self.stream.read_raw_str(inner_len);
                    self.stream.advance_raw(1); // )
                    return Ok(if is_double_colon {
                        Token::DoublePseudoClass { selector: s, value: Some(inner) }
                    } else {
                        Token::PseudoClass { selector: s, value: Some(inner) }
                    });
                } else {
                    return Ok(if is_double_colon {
                        Token::DoublePseudoClass { selector: s, value: None }
                    } else {
                        Token::PseudoClass { selector: s, value: None }
                    });
                }
            },
            b'.' => {
                // Nested class selector (e.g., .inner { } inside .outer { })
                self.after_selector = true;
                self.has_at_rule = false;
                self.stream.advance_raw(1);
                let s = self.consume_ident()?;
                return Ok(Token::ClassSelector(s));
            },
            b'#' => {
                // Nested ID selector (e.g., #inner { } inside .outer { })
                self.after_selector = true;
                self.has_at_rule = false;
                self.stream.advance_raw(1);
                let s = self.consume_ident()?;
                return Ok(Token::IdSelector(s));
            },
            b'*' => {
                // Nested universal selector
                self.after_selector = true;
                self.has_at_rule = false;
                self.stream.advance_raw(1);
                self.stream.skip_spaces();
                return Ok(Token::UniversalSelector);
            },
            b'[' => {
                // Nested attribute selector
                self.after_selector = true;
                self.has_at_rule = false;
                self.stream.advance_raw(1);
                let len = self.stream.length_to(b']')?;
                let s = self.stream.read_raw_str(len);
                self.stream.advance_raw(1); // ]
                self.stream.skip_spaces();
                return Ok(Token::AttributeSelector(s));
            },
            b'>' => {
                // Direct child combinator in nested context
                self.after_selector = false;
                self.has_at_rule = false;
                self.stream.advance_raw(1);
                self.stream.skip_spaces();
                return Ok(Token::Combinator(Combinator::GreaterThan));
            },
            b'+' => {
                // Adjacent sibling combinator in nested context
                self.after_selector = false;
                self.has_at_rule = false;
                self.stream.advance_raw(1);
                self.stream.skip_spaces();
                return Ok(Token::Combinator(Combinator::Plus));
            },
            b'~' => {
                // General sibling combinator in nested context
                self.after_selector = false;
                self.has_at_rule = false;
                self.stream.advance_raw(1);
                self.stream.skip_spaces();
                return Ok(Token::Combinator(Combinator::Tilde));
            },
            b',' => {
                // Comma in nested context (multiple selectors)
                self.after_selector = false;
                self.has_at_rule = false;
                self.stream.advance_raw(1);
                self.stream.skip_spaces();
                return Ok(Token::Comma);
            },
            b'(' if self.has_at_rule => {
                // Parenthesized content in nested @-rule
                let s = self.consume_parenthesized_content()?;
                self.after_selector = true;
                return Ok(Token::AtStr(s));
            },
            b'/' => {
                if self.consume_comment()? {
                    return self.parse_next();
                } else {
                    return Err(Error::UnknownToken(self.stream.gen_error_pos()));
                }
            }
            _ => {
                // Check for @-rule content (identifier after @rule like "@media screen")
                if self.has_at_rule {
                    let s = self.consume_ident()?;
                    self.stream.skip_spaces();
                    self.after_selector = true;
                    return Ok(Token::AtStr(s));
                }
                
                let name = self.consume_ident()?;

                self.stream.skip_spaces();

                if self.stream.is_char_eq(b'/')? {
                    if !self.consume_comment()? {
                        return Err(Error::UnknownToken(self.stream.gen_error_pos()));
                    }
                }

                if self.stream.is_char_eq(b'{')? {
                    // This is a nested type selector (e.g., "div { }" inside ".outer { }")
                    if name.is_empty() {
                        return Err(Error::UnknownToken(self.stream.gen_error_pos()));
                    } else {
                        self.after_selector = true;
                        return Ok(Token::TypeSelector(name));
                    }
                }
                
                // Check for `:` to determine if this is a declaration
                if !self.stream.is_char_eq(b':')? {
                    // Not a declaration, might be a type selector in nested context
                    self.after_selector = true;
                    return Ok(Token::TypeSelector(name));
                }

                self.stream.advance_raw(1); // :
                self.stream.skip_spaces();

                if self.stream.is_char_eq(b'/')? {
                    if !self.consume_comment()? {
                        return Err(Error::UnknownToken(self.stream.gen_error_pos()));
                    }
                }

                let len = self.stream.length_to_either(&[b';', b'}'])?;

                if len == 0 {
                    return Err(Error::UnknownToken(self.stream.gen_error_pos()));
                }

                let mut value = self.stream.read_raw_str(len);
                // trim spaces at the end of the value
                if let Some(p) = value.as_bytes().iter().rposition(|c| !stream::is_space(*c)) {
                    value = &value[0..(p + 1)];
                }

                self.stream.skip_spaces();
                while self.stream.is_char_eq(b';')? {
                    self.stream.advance_raw(1);
                    self.stream.skip_spaces();
                }

                Ok(Token::Declaration(name, value))
            }
        }
    }

    fn consume_ident(&mut self) -> Result<&'a str, Error> {
        let start = self.stream.pos();

        while !self.stream.at_end() {
            if self.stream.is_ident_raw() {
                self.stream.advance(1)?;
            } else {
                break;
            }
        }

        if start == self.stream.pos() {
            return Err(Error::UnknownToken(self.stream.gen_error_pos()));
        }

        let s = self.stream.slice_region_raw_str(start, self.stream.pos());
        Ok(s)
    }

    fn consume_comment(&mut self) -> Result<bool, Error>  {
        self.stream.advance_raw(1);

        if self.stream.is_char_eq(b'*')? {
            self.stream.advance_raw(1); // *

            while !self.stream.at_end() {
                let len = self.stream.length_to(b'*')?;
                self.stream.advance(len + 1)?;
                if self.stream.is_char_eq(b'/')? {
                    self.stream.advance_raw(1);
                    break;
                }
            }

            return Ok(true);
        } else {
            return Ok(false);
        }
    }
    
    /// Consumes parenthesized content like "(min-width: 800px)" or "(linux)"
    /// Handles nested parentheses correctly.
    fn consume_parenthesized_content(&mut self) -> Result<&'a str, Error> {
        if !self.stream.is_char_eq(b'(')? {
            return Err(Error::UnknownToken(self.stream.gen_error_pos()));
        }
        
        let start = self.stream.pos();
        self.stream.advance_raw(1); // consume opening (
        
        let mut depth = 1;
        
        while !self.stream.at_end() && depth > 0 {
            match self.stream.curr_char_raw() {
                b'(' => {
                    depth += 1;
                    self.stream.advance_raw(1);
                }
                b')' => {
                    depth -= 1;
                    self.stream.advance_raw(1);
                }
                b'"' | b'\'' => {
                    // Skip quoted strings
                    let quote = self.stream.curr_char_raw();
                    self.stream.advance_raw(1);
                    while !self.stream.at_end() {
                        let c = self.stream.curr_char_raw();
                        self.stream.advance_raw(1);
                        if c == quote {
                            break;
                        }
                        if c == b'\\' && !self.stream.at_end() {
                            self.stream.advance_raw(1); // skip escaped char
                        }
                    }
                }
                _ => {
                    self.stream.advance_raw(1);
                }
            }
        }
        
        if depth != 0 {
            return Err(Error::UnknownToken(self.stream.gen_error_pos()));
        }
        
        // Return content including the parentheses
        let end = self.stream.pos();
        let s = self.stream.slice_region_raw_str(start, end);
        self.stream.skip_spaces();
        Ok(s)
    }
}
