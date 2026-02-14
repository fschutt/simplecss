# Changelog

## 0.2.0 (2026-02-14)

### Breaking Changes

- In nested block contexts, identifiers followed by `{` now return `Token::TypeSelector` 
  instead of `Token::DeclarationStr` (enables CSS nesting)

### Added

- CSS nesting support for nested selectors and @-rules
- Nesting stack to track block depth and @-rule context
- Nested selectors: type selectors, class selectors, id selectors, universal 
  selectors, attribute selectors, pseudo-classes, combinators (` `, `>`, `+`, `~`), 
  and comma-separated selector lists inside block bodies
- Nested @-rules: `@media`, `@supports`, etc. can now appear inside rule blocks
- Parenthesized content parsing for @-rule conditions (e.g., `@media (min-width: 800px)`)
- Proper handling of nested parentheses and quoted strings inside @-rule conditions
- `consume_parenthesized_content()` method for robust parenthesized expression parsing

### Changed

- Updated to Rust 2021 edition
- Replaced deprecated `try!()` macro with `?` operator throughout
- Fixed deprecated lifetime elision patterns (`Tokenizer` → `Tokenizer<'_>`)
- Added doc comments to `PseudoClass` and `DoublePseudoClass` struct fields
- Updated repository URL to `https://github.com/fschutt/simplecss`
- Updated crate description to mention CSS nesting support

## 0.1.2

- Added tilde (`~`) combinator support

## 0.1.1

- Added preliminary parsing for inner @keyframe values
- Added parsing for `::pseudo-selector`
- Added parsing for @rule
- Extended `PseudoSelector` to also parse parentheses

## 0.1.0

- Initial release
