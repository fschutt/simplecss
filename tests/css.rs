// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

extern crate azul_simplecss;

use azul_simplecss::{Tokenizer, Token, Combinator, Error, ErrorPos};

macro_rules! test {
    ($name:ident, $text:expr, $( $token:expr ),*) => {
        #[test]
        fn $name() {
            let mut t = Tokenizer::new($text);
            $(
                assert_eq!(t.parse_next().unwrap(), $token);
            )*
            assert_eq!(t.parse_next().unwrap(), Token::EndOfStream);
        }
    };
}

macro_rules! test_selectors {
    ($name:ident, $text:expr, $( $token:expr ),*) => {
        #[test]
        fn $name() {
            let mut t = Tokenizer::new($text);
            $(
                assert_eq!(t.parse_next().unwrap(), $token);
            )*
            assert_eq!(t.parse_next().unwrap(), Token::BlockStart);
            assert_eq!(t.parse_next().unwrap(), Token::Declaration("color", "red"));
            assert_eq!(t.parse_next().unwrap(), Token::BlockEnd);
            assert_eq!(t.parse_next().unwrap(), Token::EndOfStream);
        }
    };
}

macro_rules! test_err {
    ($name:ident, $text:expr, $err:expr) => {
        #[test]
        fn $name() {
            let mut t = Tokenizer::new($text);
            assert_eq!(t.parse_next().unwrap_err(), $err);
        }
    };
}

test_selectors!(selectors_1,
    "* { color: red }",
    Token::UniversalSelector
);

test_selectors!(selectors_2,
    "p { color: red }",
    Token::TypeSelector("p")
);

test_selectors!(selectors_3,
    ":first-child { color: red }",
    Token::PseudoClass { selector: "first-child", value: None }
);

test_selectors!(selectors_4,
    ":lang(fr) { color: red }",
    Token::PseudoClass { selector: "lang", value: Some("fr") }
);

test_selectors!(selectors_6,
    ".cls { color: red }",
    Token::ClassSelector("cls")
);

test_selectors!(selectors_7,
    "#p2 { color: red }",
    Token::IdSelector("p2")
);

test_selectors!(selectors_8,
    "#p2{color:red}",
    Token::IdSelector("p2")
);

test_selectors!(selectors_9,
    " div { color:red }",
    Token::TypeSelector("div")
);

test_selectors!(complex_selectors_1,
    "h1 p { color: red; }",
    Token::TypeSelector("h1"),
    Token::Combinator(Combinator::Space),
    Token::TypeSelector("p")
);

test_selectors!(complex_selectors_2,
    "h1 p g k { color: red; }",
    Token::TypeSelector("h1"),
    Token::Combinator(Combinator::Space),
    Token::TypeSelector("p"),
    Token::Combinator(Combinator::Space),
    Token::TypeSelector("g"),
    Token::Combinator(Combinator::Space),
    Token::TypeSelector("k")
);

test_selectors!(complex_selectors_3,
    "[rel=\"author\"], [rel=\"alternate\"] { color: red; }",
    Token::AttributeSelector("rel=\"author\""),
    Token::Comma,
    Token::AttributeSelector("rel=\"alternate\"")
);

test_selectors!(complex_selectors_4,
    "div:after, div:before { color: red; }",
    Token::TypeSelector("div"),
    Token::PseudoClass { selector: "after", value: None },
    Token::Comma,
    Token::TypeSelector("div"),
    Token::PseudoClass { selector: "before", value: None }
);

test_selectors!(complex_selectors_5,
    "p.valid { color: red; }",
    Token::TypeSelector("p"),
    Token::ClassSelector("valid")
);

test_selectors!(complex_selectors_6,
    ".test:first-letter { color: red; }",
    Token::ClassSelector("test"),
    Token::PseudoClass { selector: "first-letter", value: None }
);

test_selectors!(complex_selectors_7,
    ".test, .control { color: red; }",
    Token::ClassSelector("test"),
    Token::Comma,
    Token::ClassSelector("control")
);

test_selectors!(complex_selectors_8,
    "div>h1 { color: red; }",
    Token::TypeSelector("div"),
    Token::Combinator(Combinator::GreaterThan),
    Token::TypeSelector("h1")
);

test_selectors!(complex_selectors_9,
    "div > h1 { color: red; }",
    Token::TypeSelector("div"),
    Token::Combinator(Combinator::GreaterThan),
    Token::TypeSelector("h1")
);

test_selectors!(complex_selectors_10,
    "div+h1 { color: red; }",
    Token::TypeSelector("div"),
    Token::Combinator(Combinator::Plus),
    Token::TypeSelector("h1")
);

test_selectors!(complex_selectors_11,
    "div+h1 { color: red; }",
    Token::TypeSelector("div"),
    Token::Combinator(Combinator::Plus),
    Token::TypeSelector("h1")
);

test_selectors!(complex_selectors_12,
    "p.test:first-letter { color: red; }",
    Token::TypeSelector("p"),
    Token::ClassSelector("test"),
    Token::PseudoClass { selector: "first-letter", value: None }
);

test_selectors!(complex_selectors_13,
    "#div1
+
p { color: red; }",
    Token::IdSelector("div1"),
    Token::Combinator(Combinator::Plus),
    Token::TypeSelector("p")
);

test_selectors!(complex_selectors_14,
    "button[type=\"submit\"] { color: red; }",
    Token::TypeSelector("button"),
    Token::AttributeSelector("type=\"submit\"")
);

test_selectors!(complex_selectors_15,
    "div em[id] { color: red; }",
    Token::TypeSelector("div"),
    Token::Combinator(Combinator::Space),
    Token::TypeSelector("em"),
    Token::AttributeSelector("id")
);

test_selectors!(complex_selectors_16,
    "div * em { color: red; }",
    Token::TypeSelector("div"),
    Token::UniversalSelector,
    Token::TypeSelector("em")
);

test_selectors!(complex_selectors_17,
    "div#div1 { color: red; }",
    Token::TypeSelector("div"),
    Token::IdSelector("div1")
);

test_selectors!(complex_selectors_18,
    "div#x:first-letter { color: red; }",
    Token::TypeSelector("div"),
    Token::IdSelector("x"),
    Token::PseudoClass { selector: "first-letter", value: None }
);

test_selectors!(complex_selectors_19,
    "[class=foo] + div + div + div + div { color: red; }",
    Token::AttributeSelector("class=foo"),
    Token::Combinator(Combinator::Plus),
    Token::TypeSelector("div"),
    Token::Combinator(Combinator::Plus),
    Token::TypeSelector("div"),
    Token::Combinator(Combinator::Plus),
    Token::TypeSelector("div"),
    Token::Combinator(Combinator::Plus),
    Token::TypeSelector("div")
);

test_selectors!(complex_selectors_20,
    "input[type=\"radio\"]:focus + label { color: red; }",
    Token::TypeSelector("input"),
    Token::AttributeSelector("type=\"radio\""),
    Token::PseudoClass { selector: "focus", value: None },
    Token::Combinator(Combinator::Plus),
    Token::TypeSelector("label")
);

test_selectors!(complex_selectors_21,
    ":visited:active { color: red; }",
    Token::PseudoClass { selector: "visited", value: None },
    Token::PseudoClass { selector: "active", value: None }
);

// it's actually invalid, but we do not validate it
test_selectors!(complex_selectors_22,
    "p:first-line p, #p1 { color: red; }",
    Token::TypeSelector("p"),
    Token::PseudoClass { selector: "first-line", value: None },
    Token::Combinator(Combinator::Space),
    Token::TypeSelector("p"),
    Token::Comma,
    Token::IdSelector("p1")
);

test_selectors!(complex_selectors_23,
    "p * { color: red; }",
    Token::TypeSelector("p"),
    Token::UniversalSelector
);

test_selectors!(complex_selectors_24,
    "*:active { color: red; }",
    Token::UniversalSelector,
    Token::PseudoClass { selector: "active", value: None }
);

test_selectors!(complex_selectors_25,
    "html > body > *:first-line  { color: red; }",
    Token::TypeSelector("html"),
    Token::Combinator(Combinator::GreaterThan),
    Token::TypeSelector("body"),
    Token::Combinator(Combinator::GreaterThan),
    Token::UniversalSelector,
    Token::PseudoClass { selector: "first-line", value: None }
);

test_selectors!(complex_selectors_26,
    "@keyframes mymove { color: red }",
    Token::AtRule("keyframes"),
    Token::AtStr("mymove")
);

test_selectors!(attribute_selector_1,
    "[attr=\"test\"] { color: red }",
    Token::AttributeSelector("attr=\"test\"")
);

test_selectors!(attribute_selector_2,
    "[attr=\"test\"][attr2=\"test2\"] { color: red }",
    Token::AttributeSelector("attr=\"test\""),
    Token::AttributeSelector("attr2=\"test2\"")
);

test!(blocks_1,
"p { color: red; }
p { color: red; }",
    Token::TypeSelector("p"),
    Token::BlockStart,
    Token::Declaration("color", "red"),
    Token::BlockEnd,
    Token::TypeSelector("p"),
    Token::BlockStart,
    Token::Declaration("color", "red"),
    Token::BlockEnd
);

test!(blocks_2,
"p{color:red;}p{color:red;}",
    Token::TypeSelector("p"),
    Token::BlockStart,
    Token::Declaration("color", "red"),
    Token::BlockEnd,
    Token::TypeSelector("p"),
    Token::BlockStart,
    Token::Declaration("color", "red"),
    Token::BlockEnd
);

test!(blocks_3,
"p {
    color:red;
}",
    Token::TypeSelector("p"),
    Token::BlockStart,
    Token::Declaration("color", "red"),
    Token::BlockEnd
);

test!(blocks_4,
"p
{
    color:red;
}",
    Token::TypeSelector("p"),
    Token::BlockStart,
    Token::Declaration("color", "red"),
    Token::BlockEnd
);

test!(blocks_5,
    "p{}",
    Token::TypeSelector("p"),
    Token::BlockStart,
    Token::BlockEnd
);

test!(blocks_6,
    "@keyframes hello { from { width: 500px; } to { width: 600px; } }",
    Token::AtRule("keyframes"),
    Token::AtStr("hello"),
    Token::BlockStart,
    // With nesting support, 'from' and 'to' are now parsed as TypeSelector (like nested selectors)
    Token::TypeSelector("from"),
    Token::BlockStart,
    Token::Declaration("width", "500px"),
    Token::BlockEnd,
    Token::TypeSelector("to"),
    Token::BlockStart,
    Token::Declaration("width", "600px"),
    Token::BlockEnd,
    Token::BlockEnd
);

#[test]
fn declarations_1() {
    let vec = vec![
        "p {color:red}",
        "p {color:red;}",
        "p {color:red }",
        "p { color: red; }",
        "p { color : red ; }",
        "p {  color  :  red  ;  } ",
        "p { color : red ; }"
    ];

    for css in vec {
        let mut t = Tokenizer::new(css);
        assert_eq!(t.parse_next().unwrap(), Token::TypeSelector("p"));
        assert_eq!(t.parse_next().unwrap(), Token::BlockStart);
        assert_eq!(t.parse_next().unwrap(), Token::Declaration("color", "red"));
        assert_eq!(t.parse_next().unwrap(), Token::BlockEnd);
        assert_eq!(t.parse_next().unwrap(), Token::EndOfStream);
    }
}

test!(declarations_2,
    "p { color:red;;;;color:red; }",
    Token::TypeSelector("p"),
    Token::BlockStart,
    Token::Declaration("color", "red"),
    Token::Declaration("color", "red"),
    Token::BlockEnd
);

test!(declarations_3,
    "* {list-style-image: url(\"img.png\");}",
    Token::UniversalSelector,
    Token::BlockStart,
    Token::Declaration("list-style-image", "url(\"img.png\")"),
    Token::BlockEnd
);

test!(declarations_4,
    "* { color: white ! important; }",
    Token::UniversalSelector,
    Token::BlockStart,
    Token::Declaration("color", "white ! important"),
    Token::BlockEnd
);

test!(declarations_5,
    "* { border: 1em solid blue; background: navy url(support/diamond.png) -2em -2em no-repeat }",
    Token::UniversalSelector,
    Token::BlockStart,
    Token::Declaration("border", "1em solid blue"),
    Token::Declaration("background", "navy url(support/diamond.png) -2em -2em no-repeat"),
    Token::BlockEnd
);

test!(declarations_6,
    "* {stroke-width:2}",
    Token::UniversalSelector,
    Token::BlockStart,
    Token::Declaration("stroke-width", "2"),
    Token::BlockEnd
);

test!(comment_1,
    "/* .test { color: green ! important; } */
    * { color: red; }",
    Token::UniversalSelector,
    Token::BlockStart,
    Token::Declaration("color", "red"),
    Token::BlockEnd
);

test!(comment_2,
    "p /* comment */ { color:red }",
    Token::TypeSelector("p"),
    Token::BlockStart,
    Token::Declaration("color", "red"),
    Token::BlockEnd
);

test!(comment_3,
    "p /* comment */ div { color:red }",
    Token::TypeSelector("p"),
    Token::Combinator(Combinator::Space),
    Token::TypeSelector("div"),
    Token::BlockStart,
    Token::Declaration("color", "red"),
    Token::BlockEnd
);

test!(comment_4,
    "div { /**/color: red; }",
    Token::TypeSelector("div"),
    Token::BlockStart,
    Token::Declaration("color", "red"),
    Token::BlockEnd
);

test!(comment_5,
    "div { /**/color: red; }",
    Token::TypeSelector("div"),
    Token::BlockStart,
    Token::Declaration("color", "red"),
    Token::BlockEnd
);

test!(comment_6,
    "div { /* *\\/*/color: red; }",
    Token::TypeSelector("div"),
    Token::BlockStart,
    Token::Declaration("color", "red"),
    Token::BlockEnd
);

// TODO: comment can be between 'red' and ';'
test!(comment_7,
"/*Comment*/div/*Comment*/
{
  /*Comment*/color/*Comment*/: /*Comment*/red;
  /*Comment*/
}/*Comment*/",
    Token::TypeSelector("div"),
    Token::BlockStart,
    Token::Declaration("color", "red"),
    Token::BlockEnd
);

test!(comment_8,
" /*
   * Comment
   */
  div
 {
    color : red
 }",
    Token::TypeSelector("div"),
    Token::BlockStart,
    Token::Declaration("color", "red"),
    Token::BlockEnd
);

test_err!(invalid_2,
    "# div1",
    Error::UnknownToken(ErrorPos::new(1, 2))
);

// test_err!(invalid_3,
//     "#1div ",
//     Error::UnknownToken(ErrorPos::new(1, 2))
// );

test!(invalid_4,
    "@import",
    Token::AtRule("import")
);

#[test]
fn invalid_5() {
    let mut t = Tokenizer::new("div { {color: red;} }");
    assert_eq!(t.parse_next().unwrap(), Token::TypeSelector("div"));
    assert_eq!(t.parse_next().unwrap(), Token::BlockStart);
    assert_eq!(t.parse_next().unwrap(), Token::BlockStart); // wrong!
    // assert_eq!(t.parse_next().unwrap_err(), Error::UnknownToken(ErrorPos::new(1, 7)));
}

#[test]
fn invalid_6() {
    let mut t = Tokenizer::new("div { (color: red;) }");
    assert_eq!(t.parse_next().unwrap(), Token::TypeSelector("div"));
    assert_eq!(t.parse_next().unwrap(), Token::BlockStart);
    assert_eq!(t.parse_next().unwrap_err(), Error::UnknownToken(ErrorPos::new(1, 7)));
}

#[test]
fn invalid_7() {
    // With nesting support, [ is now parsed as an attribute selector start,
    // so we get an AttributeSelector token (with invalid content including the semicolon)
    let mut t = Tokenizer::new("div { [color: red;] }");
    assert_eq!(t.parse_next().unwrap(), Token::TypeSelector("div"));
    assert_eq!(t.parse_next().unwrap(), Token::BlockStart);
    // The attribute selector now captures everything until ]
    assert_eq!(t.parse_next().unwrap(), Token::AttributeSelector("color: red;"));
    assert_eq!(t.parse_next().unwrap(), Token::BlockEnd);
    assert_eq!(t.parse_next().unwrap(), Token::EndOfStream);
}

#[test]
fn invalid_8() {
    let mut t = Tokenizer::new("div { color: }");
    assert_eq!(t.parse_next().unwrap(), Token::TypeSelector("div"));
    assert_eq!(t.parse_next().unwrap(), Token::BlockStart);
    // assert_eq!(t.parse_next().unwrap(), Token::Property("color"));
    assert_eq!(t.parse_next().unwrap_err(), Error::UnknownToken(ErrorPos::new(1, 14)));
}

#[test]
fn invalid_9() {
    let mut t = Tokenizer::new("div");
    assert_eq!(t.parse_next().unwrap(), Token::TypeSelector("div"));
    // TODO: should be UnexpectedEndOfStream
    assert_eq!(t.parse_next().unwrap(), Token::EndOfStream);
}

#[test]
fn invalid_10() {
    let mut t = Tokenizer::new("div { /\\*;color: green;*/ }");
    assert_eq!(t.parse_next().unwrap(), Token::TypeSelector("div"));
    assert_eq!(t.parse_next().unwrap(), Token::BlockStart);
    assert_eq!(t.parse_next().unwrap_err(), Error::UnknownToken(ErrorPos::new(1, 8)));
}

#[test]
fn invalid_11() {
    // With nesting support, after the comment /*\*/ the * is parsed as UniversalSelector
    let mut t = Tokenizer::new("div { /*\\*/*/color: red; }");
    assert_eq!(t.parse_next().unwrap(), Token::TypeSelector("div"));
    assert_eq!(t.parse_next().unwrap(), Token::BlockStart);
    // Comment /*\*/ is consumed, then * is a UniversalSelector, then / causes error
    assert_eq!(t.parse_next().unwrap(), Token::UniversalSelector);
    assert_eq!(t.parse_next().unwrap_err(), Error::UnknownToken(ErrorPos::new(1, 14)));
}

test_err!(invalid_12,
    ".平和 { color: red; }",
    Error::UnknownToken(ErrorPos::new(1, 2))
);

// #[test]
// fn invalid_13() {
//     let mut t = Tokenizer::new("div { causta: \"}\" + ({7} * '\'') }");
//     assert_eq!(t.parse_next().unwrap(), Token::TypeSelector("div"));
//     assert_eq!(t.parse_next().unwrap(), Token::BlockStart);
//     assert_eq!(t.parse_next().unwrap(), Token::Declaration("causta", "\""));
//     assert_eq!(t.parse_next().unwrap_err(), Error::UnknownToken(ErrorPos::new(1, 15)));
// }

#[test]
fn invalid_14() {
    let mut t = Tokenizer::new(
"div
{
    \"this is a string]}\"\"[{\\\"'\";  /*should be parsed as a string but be ignored*/
    {{}}[]'';                     /*should be parsed as nested blocks and a string but be ignored*/
    color: green;
}");
    assert_eq!(t.parse_next().unwrap(), Token::TypeSelector("div"));
    assert_eq!(t.parse_next().unwrap(), Token::BlockStart);
    assert_eq!(t.parse_next().unwrap_err(), Error::UnknownToken(ErrorPos::new(3, 5)));
}

test_err!(invalid_15,
    ".\\xC3\\xA9 { color: red; }",
    Error::UnknownToken(ErrorPos::new(1, 2))
);

test!(invalid_16,
    "::invalidPseudoElement",
    Token::DoublePseudoClass { selector: "invalidPseudoElement", value: None }
);

#[test]
fn invalid_17() {
    let mut t = Tokenizer::new(" ");
    assert_eq!(t.parse_next().unwrap(), Token::EndOfStream);
}

#[test]
fn invalid_18() {
    let mut t = Tokenizer::new("div > >");
    assert_eq!(t.parse_next().unwrap(), Token::TypeSelector("div"));
    assert_eq!(t.parse_next().unwrap(), Token::Combinator(Combinator::GreaterThan));
    assert_eq!(t.parse_next().unwrap_err(), Error::UnknownToken(ErrorPos::new(1, 7)));
}

// =====================================================================
// CSS NESTING TESTS
// =====================================================================

/// Test nested pseudo-class selector: .button { :hover { color: red; } }
#[test]
fn nested_pseudo_class() {
    let mut t = Tokenizer::new(".button { :hover { color: red; } }");
    assert_eq!(t.parse_next().unwrap(), Token::ClassSelector("button"));
    assert_eq!(t.parse_next().unwrap(), Token::BlockStart);
    assert_eq!(t.parse_next().unwrap(), Token::PseudoClass { selector: "hover", value: None });
    assert_eq!(t.parse_next().unwrap(), Token::BlockStart);
    assert_eq!(t.parse_next().unwrap(), Token::Declaration("color", "red"));
    assert_eq!(t.parse_next().unwrap(), Token::BlockEnd);
    assert_eq!(t.parse_next().unwrap(), Token::BlockEnd);
    assert_eq!(t.parse_next().unwrap(), Token::EndOfStream);
}

/// Test nested class selector: .outer { .inner { color: blue; } }
#[test]
fn nested_class_selector() {
    let mut t = Tokenizer::new(".outer { .inner { color: blue; } }");
    assert_eq!(t.parse_next().unwrap(), Token::ClassSelector("outer"));
    assert_eq!(t.parse_next().unwrap(), Token::BlockStart);
    assert_eq!(t.parse_next().unwrap(), Token::ClassSelector("inner"));
    assert_eq!(t.parse_next().unwrap(), Token::BlockStart);
    assert_eq!(t.parse_next().unwrap(), Token::Declaration("color", "blue"));
    assert_eq!(t.parse_next().unwrap(), Token::BlockEnd);
    assert_eq!(t.parse_next().unwrap(), Token::BlockEnd);
    assert_eq!(t.parse_next().unwrap(), Token::EndOfStream);
}

/// Test nested ID selector: .outer { #inner { color: green; } }
#[test]
fn nested_id_selector() {
    let mut t = Tokenizer::new(".outer { #inner { color: green; } }");
    assert_eq!(t.parse_next().unwrap(), Token::ClassSelector("outer"));
    assert_eq!(t.parse_next().unwrap(), Token::BlockStart);
    assert_eq!(t.parse_next().unwrap(), Token::IdSelector("inner"));
    assert_eq!(t.parse_next().unwrap(), Token::BlockStart);
    assert_eq!(t.parse_next().unwrap(), Token::Declaration("color", "green"));
    assert_eq!(t.parse_next().unwrap(), Token::BlockEnd);
    assert_eq!(t.parse_next().unwrap(), Token::BlockEnd);
    assert_eq!(t.parse_next().unwrap(), Token::EndOfStream);
}

/// Test nested @-rule: .button { @os linux { background: blue; } }
#[test]
fn nested_at_rule() {
    let mut t = Tokenizer::new(".button { @os linux { background: blue; } }");
    assert_eq!(t.parse_next().unwrap(), Token::ClassSelector("button"));
    assert_eq!(t.parse_next().unwrap(), Token::BlockStart);
    assert_eq!(t.parse_next().unwrap(), Token::AtRule("os"));
    assert_eq!(t.parse_next().unwrap(), Token::AtStr("linux"));
    assert_eq!(t.parse_next().unwrap(), Token::BlockStart);
    assert_eq!(t.parse_next().unwrap(), Token::Declaration("background", "blue"));
    assert_eq!(t.parse_next().unwrap(), Token::BlockEnd);
    assert_eq!(t.parse_next().unwrap(), Token::BlockEnd);
    assert_eq!(t.parse_next().unwrap(), Token::EndOfStream);
}

/// Test nested @media with parenthesized condition
#[test]
fn nested_at_media() {
    let mut t = Tokenizer::new(".container { @media (min-width: 800px) { font-size: 18px; } }");
    assert_eq!(t.parse_next().unwrap(), Token::ClassSelector("container"));
    assert_eq!(t.parse_next().unwrap(), Token::BlockStart);
    assert_eq!(t.parse_next().unwrap(), Token::AtRule("media"));
    assert_eq!(t.parse_next().unwrap(), Token::AtStr("(min-width: 800px)"));
    assert_eq!(t.parse_next().unwrap(), Token::BlockStart);
    assert_eq!(t.parse_next().unwrap(), Token::Declaration("font-size", "18px"));
    assert_eq!(t.parse_next().unwrap(), Token::BlockEnd);
    assert_eq!(t.parse_next().unwrap(), Token::BlockEnd);
    assert_eq!(t.parse_next().unwrap(), Token::EndOfStream);
}

/// Test multiple nesting levels: .a { .b { .c { color: red; } } }
#[test]
fn deeply_nested_selectors() {
    let mut t = Tokenizer::new(".a { .b { .c { color: red; } } }");
    assert_eq!(t.parse_next().unwrap(), Token::ClassSelector("a"));
    assert_eq!(t.parse_next().unwrap(), Token::BlockStart);
    assert_eq!(t.parse_next().unwrap(), Token::ClassSelector("b"));
    assert_eq!(t.parse_next().unwrap(), Token::BlockStart);
    assert_eq!(t.parse_next().unwrap(), Token::ClassSelector("c"));
    assert_eq!(t.parse_next().unwrap(), Token::BlockStart);
    assert_eq!(t.parse_next().unwrap(), Token::Declaration("color", "red"));
    assert_eq!(t.parse_next().unwrap(), Token::BlockEnd);
    assert_eq!(t.parse_next().unwrap(), Token::BlockEnd);
    assert_eq!(t.parse_next().unwrap(), Token::BlockEnd);
    assert_eq!(t.parse_next().unwrap(), Token::EndOfStream);
}

/// Test mixed declarations and nested rules
#[test]
fn mixed_declarations_and_nested() {
    let mut t = Tokenizer::new(".button { color: blue; :hover { color: red; } background: white; }");
    assert_eq!(t.parse_next().unwrap(), Token::ClassSelector("button"));
    assert_eq!(t.parse_next().unwrap(), Token::BlockStart);
    assert_eq!(t.parse_next().unwrap(), Token::Declaration("color", "blue"));
    assert_eq!(t.parse_next().unwrap(), Token::PseudoClass { selector: "hover", value: None });
    assert_eq!(t.parse_next().unwrap(), Token::BlockStart);
    assert_eq!(t.parse_next().unwrap(), Token::Declaration("color", "red"));
    assert_eq!(t.parse_next().unwrap(), Token::BlockEnd);
    assert_eq!(t.parse_next().unwrap(), Token::Declaration("background", "white"));
    assert_eq!(t.parse_next().unwrap(), Token::BlockEnd);
    assert_eq!(t.parse_next().unwrap(), Token::EndOfStream);
}

/// Test nested universal selector
#[test]
fn nested_universal_selector() {
    let mut t = Tokenizer::new(".container { * { margin: 0; } }");
    assert_eq!(t.parse_next().unwrap(), Token::ClassSelector("container"));
    assert_eq!(t.parse_next().unwrap(), Token::BlockStart);
    assert_eq!(t.parse_next().unwrap(), Token::UniversalSelector);
    assert_eq!(t.parse_next().unwrap(), Token::BlockStart);
    assert_eq!(t.parse_next().unwrap(), Token::Declaration("margin", "0"));
    assert_eq!(t.parse_next().unwrap(), Token::BlockEnd);
    assert_eq!(t.parse_next().unwrap(), Token::BlockEnd);
    assert_eq!(t.parse_next().unwrap(), Token::EndOfStream);
}

/// Test nested attribute selector
#[test]
fn nested_attribute_selector() {
    let mut t = Tokenizer::new(".form { [type=\"text\"] { border: 1px; } }");
    assert_eq!(t.parse_next().unwrap(), Token::ClassSelector("form"));
    assert_eq!(t.parse_next().unwrap(), Token::BlockStart);
    assert_eq!(t.parse_next().unwrap(), Token::AttributeSelector("type=\"text\""));
    assert_eq!(t.parse_next().unwrap(), Token::BlockStart);
    assert_eq!(t.parse_next().unwrap(), Token::Declaration("border", "1px"));
    assert_eq!(t.parse_next().unwrap(), Token::BlockEnd);
    assert_eq!(t.parse_next().unwrap(), Token::BlockEnd);
    assert_eq!(t.parse_next().unwrap(), Token::EndOfStream);
}

/// Test nested type selector: .container { div { color: red; } }
#[test]
fn nested_type_selector() {
    let mut t = Tokenizer::new(".container { div { color: red; } }");
    assert_eq!(t.parse_next().unwrap(), Token::ClassSelector("container"));
    assert_eq!(t.parse_next().unwrap(), Token::BlockStart);
    assert_eq!(t.parse_next().unwrap(), Token::TypeSelector("div"));
    assert_eq!(t.parse_next().unwrap(), Token::BlockStart);
    assert_eq!(t.parse_next().unwrap(), Token::Declaration("color", "red"));
    assert_eq!(t.parse_next().unwrap(), Token::BlockEnd);
    assert_eq!(t.parse_next().unwrap(), Token::BlockEnd);
    assert_eq!(t.parse_next().unwrap(), Token::EndOfStream);
}

/// Test nested combinator: .parent { > .child { color: red; } }
#[test]
fn nested_direct_child_combinator() {
    let mut t = Tokenizer::new(".parent { > .child { color: red; } }");
    assert_eq!(t.parse_next().unwrap(), Token::ClassSelector("parent"));
    assert_eq!(t.parse_next().unwrap(), Token::BlockStart);
    assert_eq!(t.parse_next().unwrap(), Token::Combinator(Combinator::GreaterThan));
    assert_eq!(t.parse_next().unwrap(), Token::ClassSelector("child"));
    assert_eq!(t.parse_next().unwrap(), Token::BlockStart);
    assert_eq!(t.parse_next().unwrap(), Token::Declaration("color", "red"));
    assert_eq!(t.parse_next().unwrap(), Token::BlockEnd);
    assert_eq!(t.parse_next().unwrap(), Token::BlockEnd);
    assert_eq!(t.parse_next().unwrap(), Token::EndOfStream);
}

/// Test nested ::pseudo-element
#[test]
fn nested_pseudo_element() {
    let mut t = Tokenizer::new(".button { ::before { content: ''; } }");
    assert_eq!(t.parse_next().unwrap(), Token::ClassSelector("button"));
    assert_eq!(t.parse_next().unwrap(), Token::BlockStart);
    assert_eq!(t.parse_next().unwrap(), Token::DoublePseudoClass { selector: "before", value: None });
    assert_eq!(t.parse_next().unwrap(), Token::BlockStart);
    assert_eq!(t.parse_next().unwrap(), Token::Declaration("content", "''"));
    assert_eq!(t.parse_next().unwrap(), Token::BlockEnd);
    assert_eq!(t.parse_next().unwrap(), Token::BlockEnd);
    assert_eq!(t.parse_next().unwrap(), Token::EndOfStream);
}

/// Test @os at top level (not nested)
#[test]
fn at_os_top_level() {
    let mut t = Tokenizer::new("@os linux { .button { color: red; } }");
    assert_eq!(t.parse_next().unwrap(), Token::AtRule("os"));
    assert_eq!(t.parse_next().unwrap(), Token::AtStr("linux"));
    assert_eq!(t.parse_next().unwrap(), Token::BlockStart);
    assert_eq!(t.parse_next().unwrap(), Token::ClassSelector("button"));
    assert_eq!(t.parse_next().unwrap(), Token::BlockStart);
    assert_eq!(t.parse_next().unwrap(), Token::Declaration("color", "red"));
    assert_eq!(t.parse_next().unwrap(), Token::BlockEnd);
    assert_eq!(t.parse_next().unwrap(), Token::BlockEnd);
    assert_eq!(t.parse_next().unwrap(), Token::EndOfStream);
}

/// Test multiple nested selectors with comma
#[test]
fn nested_comma_selectors() {
    let mut t = Tokenizer::new(".parent { :hover, :focus { color: red; } }");
    assert_eq!(t.parse_next().unwrap(), Token::ClassSelector("parent"));
    assert_eq!(t.parse_next().unwrap(), Token::BlockStart);
    assert_eq!(t.parse_next().unwrap(), Token::PseudoClass { selector: "hover", value: None });
    assert_eq!(t.parse_next().unwrap(), Token::Comma);
    assert_eq!(t.parse_next().unwrap(), Token::PseudoClass { selector: "focus", value: None });
    assert_eq!(t.parse_next().unwrap(), Token::BlockStart);
    assert_eq!(t.parse_next().unwrap(), Token::Declaration("color", "red"));
    assert_eq!(t.parse_next().unwrap(), Token::BlockEnd);
    assert_eq!(t.parse_next().unwrap(), Token::BlockEnd);
    assert_eq!(t.parse_next().unwrap(), Token::EndOfStream);
}
