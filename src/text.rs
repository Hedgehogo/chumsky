//! Text-specific parsers and utilities.
//!
//! *“Ford!" he said, "there's an infinite number of monkeys outside who want to talk to us about this script for
//! Hamlet they've worked out.”*
//!
//! The parsers in this module are generic over both Unicode ([`char`]) and ASCII ([`u8`]) characters. Most parsers take
//! a type parameter, `C`, that can be either [`u8`] or [`char`] in order to handle either case.

use crate::prelude::*;

use super::*;

/// A trait implemented by textual character types (currently, [`u8`] and [`char`]).
///
/// This trait is currently sealed to minimize the impact of breaking changes. If you find a type that you think should
/// implement this trait, please [open an issue/PR](https://github.com/zesterer/chumsky/issues/new).
pub trait Char: Copy + PartialEq + Sealed {
    /// Returns true if the character is canonically considered to be inline whitespace (i.e: not part of a newline).
    fn is_inline_whitespace(&self) -> bool;

    /// Returns true if the character is canonically considered to be whitespace.
    fn is_whitespace(&self) -> bool;

    /// Returns true if the character is canonically considered to be newline.
    fn is_newline(&self) -> bool;

    /// Return the '0' digit of the character.
    fn digit_zero() -> Self;

    /// Returns true if the character is canonically considered to be a numeric digit.
    fn is_digit(&self, radix: u32) -> bool;

    /// Returns true if the character is canonically considered to be valid for starting an identifier.
    fn is_ident_start(&self) -> bool;

    /// Returns true if the character is canonically considered to be a valid within an identifier.
    fn is_ident_continue(&self) -> bool;

    /// Returns this character as a [`char`].
    fn to_ascii(&self) -> Option<u8>;
}

impl<'src> Sealed for Grapheme<'src> {}
impl<'src> Char for Grapheme<'src> {
    fn is_inline_whitespace(&self) -> bool {
        self.as_str() == " " || self.as_str() == "\t"
    }

    fn is_whitespace(&self) -> bool {
        let mut iter = self.as_str().chars();
        iter.all(char::is_whitespace)
    }

    fn is_newline(&self) -> bool {
        [
            "\r\n",     // CR LF
            "\n",       // Newline
            "\r",       // Carriage return
            "\x0B",     // Vertical tab
            "\x0C",     // Form feed
            "\u{0085}", // Next line
            "\u{2028}", // Line separator
            "\u{2029}", // Paragraph separator
        ]
        .as_slice()
        .contains(&self.as_str())
    }

    fn digit_zero() -> Self {
        Grapheme::digit_zero()
    }

    fn is_digit(&self, radix: u32) -> bool {
        let mut iter = self.as_str().chars();
        match (iter.next(), iter.next()) {
            (Some(i), None) => i.is_digit(radix),
            _ => false,
        }
    }

    fn to_ascii(&self) -> Option<u8> {
        let mut iter = self.as_bytes().iter();
        match (iter.next(), iter.next()) {
            (Some(i), None) if i.is_ascii() => Some(*i),
            _ => None,
        }
    }

    fn is_ident_start(&self) -> bool {
        let (first, rest) = self.split();
        let is_start = unicode_ident::is_xid_start(first) || first == '_';
        is_start && rest.chars().all(|i| unicode_ident::is_xid_continue(i))
    }

    fn is_ident_continue(&self) -> bool {
        let mut iter = self.as_str().chars();
        iter.all(|i| unicode_ident::is_xid_continue(i))
    }
}

impl Sealed for char {}
impl Char for char {
    fn is_inline_whitespace(&self) -> bool {
        *self == ' ' || *self == '\t'
    }

    fn is_whitespace(&self) -> bool {
        char::is_whitespace(*self)
    }

    fn is_newline(&self) -> bool {
        [
            '\n',       // Newline
            '\r',       // Carriage return
            '\x0B',     // Vertical tab
            '\x0C',     // Form feed
            '\u{0085}', // Next line
            '\u{2028}', // Line separator
            '\u{2029}', // Paragraph separator
        ]
        .as_slice()
        .contains(self)
    }

    fn digit_zero() -> Self {
        '0'
    }

    fn is_digit(&self, radix: u32) -> bool {
        char::is_digit(*self, radix)
    }

    fn to_ascii(&self) -> Option<u8> {
        self.is_ascii().then_some(*self as u8)
    }

    fn is_ident_start(&self) -> bool {
        unicode_ident::is_xid_start(*self) || *self == '_'
    }

    fn is_ident_continue(&self) -> bool {
        unicode_ident::is_xid_continue(*self)
    }
}

impl Sealed for u8 {}
impl Char for u8 {
    fn is_inline_whitespace(&self) -> bool {
        *self == b' ' || *self == b'\t'
    }

    fn is_whitespace(&self) -> bool {
        self.is_ascii_whitespace()
    }

    fn is_newline(&self) -> bool {
        [
            b'\n',   // Newline
            b'\r',   // Carriage return
            b'\x0B', // Vertical tab
            b'\x0C', // Form feed
        ]
        .as_slice()
        .contains(self)
    }

    fn digit_zero() -> Self {
        b'0'
    }

    fn is_digit(&self, radix: u32) -> bool {
        (*self as char).is_digit(radix)
    }

    fn to_ascii(&self) -> Option<u8> {
        Some(*self)
    }

    fn is_ident_start(&self) -> bool {
        (*self as char).is_ident_start()
    }

    fn is_ident_continue(&self) -> bool {
        (*self as char).is_ident_continue()
    }
}

/// A parser that accepts (and ignores) any number of whitespace characters before or after another pattern.
#[derive(Copy, Clone)]
pub struct Padded<A> {
    pub(crate) parser: A,
}

impl<'a, I, O, E, A> ParserSealed<'a, I, O, E> for Padded<A>
where
    I: ValueInput<'a>,
    E: ParserExtra<'a, I>,
    I::Token: Char,
    A: Parser<'a, I, O, E>,
{
    fn go<M: Mode>(&self, inp: &mut InputRef<'a, '_, I, E>) -> PResult<M, O> {
        inp.skip_while(|c| c.is_whitespace());
        let out = self.parser.go::<M>(inp)?;
        inp.skip_while(|c| c.is_whitespace());
        Ok(out)
    }

    go_extra!(O);
}

/// A parser that accepts (and ignores) any number of whitespace characters.
///
/// This parser is a `Parser::Repeated` and so methods such as `at_least()` can be called on it.
///
/// The output type of this parser is `()`.
///
/// # Examples
///
/// ```
/// # use chumsky::prelude::*;
/// let whitespace = text::whitespace::<_, extra::Err<Simple<char>>>();
///
/// // Any amount of whitespace is parsed...
/// assert_eq!(whitespace.parse("\t \n  \r ").into_result(), Ok(()));
/// // ...including none at all!
/// assert_eq!(whitespace.parse("").into_result(), Ok(()));
/// ```
pub fn whitespace<'a, I, E>() -> Repeated<impl Parser<'a, I, (), E> + Copy, (), I, E>
where
    I: StrInput<'a, Token: 'a>,
    E: ParserExtra<'a, I>,
{
    select! { c if (c as I::Token).is_whitespace() => () }
        .ignored()
        .repeated()
}

/// A parser that accepts (and ignores) any number of inline whitespace characters.
///
/// This parser is a `Parser::Repeated` and so methods such as `at_least()` can be called on it.
///
/// The output type of this parser is `()`.
///
/// # Examples
///
/// ```
/// # use chumsky::prelude::*;
/// let inline_whitespace = text::inline_whitespace::<_, extra::Err<Simple<char>>>();
///
/// // Any amount of inline whitespace is parsed...
/// assert_eq!(inline_whitespace.parse("\t  ").into_result(), Ok(()));
/// // ...including none at all!
/// assert_eq!(inline_whitespace.parse("").into_result(), Ok(()));
/// // ... but not newlines
/// assert!(inline_whitespace.at_least(1).parse("\n\r").has_errors());
/// ```
pub fn inline_whitespace<'a, I, E>() -> Repeated<impl Parser<'a, I, (), E> + Copy, (), I, E>
where
    I: StrInput<'a, Token: 'a>,
    E: ParserExtra<'a, I>,
{
    select! { c if (c as I::Token).is_inline_whitespace() => () }
        .ignored()
        .repeated()
}

/// A parser that accepts (and ignores) any newline characters or character sequences.
///
/// The output type of this parser is `()`.
///
/// This parser is quite extensive, recognizing:
///
/// - Line feed (`\n`)
/// - Carriage return (`\r`)
/// - Carriage return + line feed (`\r\n`)
/// - Vertical tab (`\x0B`)
/// - Form feed (`\x0C`)
/// - Next line (`\u{0085}`)
/// - Line separator (`\u{2028}`)
/// - Paragraph separator (`\u{2029}`)
///
/// # Examples
///
/// ```
/// # use chumsky::prelude::*;
/// let newline = text::newline::<_, extra::Err<Simple<char>>>();
///
/// assert_eq!(newline.parse("\n").into_result(), Ok(()));
/// assert_eq!(newline.parse("\r").into_result(), Ok(()));
/// assert_eq!(newline.parse("\r\n").into_result(), Ok(()));
/// assert_eq!(newline.parse("\x0B").into_result(), Ok(()));
/// assert_eq!(newline.parse("\x0C").into_result(), Ok(()));
/// assert_eq!(newline.parse("\u{0085}").into_result(), Ok(()));
/// assert_eq!(newline.parse("\u{2028}").into_result(), Ok(()));
/// assert_eq!(newline.parse("\u{2029}").into_result(), Ok(()));
/// ```
#[must_use]
pub fn newline<'a, I, E>() -> impl Parser<'a, I, (), E> + Copy
where
    I: ValueInput<'a, Token: Char + 'a>,
    E: ParserExtra<'a, I>,
    &'a str: OrderedSeq<'a, I::Token>,
{
    just("\r\n")
        .ignored()
        .or(any().filter(I::Token::is_newline).ignored())
}

/// A parser that accepts one or more ASCII digits.
///
/// The output type of this parser is `I::Slice` (i.e: [`&str`] when `I` is [`&str`], and [`&[u8]`]
/// when `I::Slice` is [`&[u8]`]).
///
/// The `radix` parameter functions identically to [`char::is_digit`]. If in doubt, choose `10`.
///
/// # Examples
///
/// ```
/// # use chumsky::prelude::*;
/// let digits = text::digits::<_, extra::Err<Simple<char>>>(10).to_slice();
///
/// assert_eq!(digits.parse("0").into_result(), Ok("0"));
/// assert_eq!(digits.parse("1").into_result(), Ok("1"));
/// assert_eq!(digits.parse("01234").into_result(), Ok("01234"));
/// assert_eq!(digits.parse("98345").into_result(), Ok("98345"));
/// // A string of zeroes is still valid. Use `int` if this is not desirable.
/// assert_eq!(digits.parse("0000").into_result(), Ok("0000"));
/// assert!(digits.parse("").has_errors());
/// ```
#[must_use]
pub fn digits<'a, I, E>(
    radix: u32,
) -> Repeated<impl Parser<'a, I, I::Token, E> + Copy, I::Token, I, E>
where
    I: ValueInput<'a, Token: Char + 'a>,
    E: ParserExtra<'a, I>,
{
    any()
        // Use try_map over filter to get a better error on failure
        .try_map(move |c: I::Token, span| {
            if c.is_digit(radix) {
                Ok(c)
            } else {
                Err(Error::expected_found([], Some(MaybeRef::Val(c)), span))
            }
        })
        .repeated()
        .at_least(1)
}

/// A parser that accepts a non-negative integer.
///
/// An integer is defined as a non-empty sequence of ASCII digits, where the first digit is non-zero or the sequence
/// has length one.
///
/// The output type of this parser is `I::Slice` (i.e: [`&str`] when `I` is [`&str`], and [`&[u8]`]
/// when `I::Slice` is [`&[u8]`]).
///
/// The `radix` parameter functions identically to [`char::is_digit`]. If in doubt, choose `10`.
///
/// # Examples
///
/// ```
/// # use chumsky::prelude::*;
/// let dec = text::int::<_, extra::Err<Simple<char>>>(10);
///
/// assert_eq!(dec.parse("0").into_result(), Ok("0"));
/// assert_eq!(dec.parse("1").into_result(), Ok("1"));
/// assert_eq!(dec.parse("1452").into_result(), Ok("1452"));
/// // No leading zeroes are permitted!
/// assert!(dec.parse("04").has_errors());
///
/// let hex = text::int::<_, extra::Err<Simple<char>>>(16);
///
/// assert_eq!(hex.parse("2A").into_result(), Ok("2A"));
/// assert_eq!(hex.parse("d").into_result(), Ok("d"));
/// assert_eq!(hex.parse("b4").into_result(), Ok("b4"));
/// assert!(hex.parse("0B").has_errors());
/// ```
///
#[must_use]
pub fn int<'a, I, E>(radix: u32) -> impl Parser<'a, I, I::Slice, E> + Copy
where
    I: StrInput<'a, Token: 'a>,
    E: ParserExtra<'a, I>,
{
    any()
        // Use try_map over filter to get a better error on failure
        .try_map(move |c: I::Token, span| {
            if c.is_digit(radix) && c != I::Token::digit_zero() {
                Ok(c)
            } else {
                Err(Error::expected_found([], Some(MaybeRef::Val(c)), span))
            }
        })
        // This error never appears due to `repeated` so can use `filter`
        .then(select! { c if (c as I::Token).is_digit(radix) => () }.repeated())
        .ignored()
        .or(just(I::Token::digit_zero()).ignored())
        .to_slice()
}

/// Parsers and utilities for working with ASCII inputs.
pub mod ascii {
    use super::*;

    /// A parser that accepts a C-style identifier.
    ///
    /// The output type of this parser is [`Char::Str`] (i.e: [`&str`] when `C` is [`char`], and [`&[u8]`] when `C` is
    /// [`u8`]).
    ///
    /// An identifier is defined as an ASCII alphabetic character or an underscore followed by any number of alphanumeric
    /// characters or underscores. The regex pattern for it is `[a-zA-Z_][a-zA-Z0-9_]*`.
    #[must_use]
    pub fn ident<'a, I, E>() -> impl Parser<'a, I, I::Slice, E> + Copy
    where
        I: StrInput<'a, Token: 'a>,
        E: ParserExtra<'a, I>,
    {
        any()
            // Use try_map over filter to get a better error on failure
            .try_map(|c: I::Token, span| {
                if c.to_ascii().map(|i| i.is_ascii_alphabetic() || i == b'_').unwrap_or(false) {
                    Ok(c)
                } else {
                    Err(Error::expected_found([], Some(MaybeRef::Val(c)), span))
                }
            })
            .then(
                select! { c if (c as I::Token).to_ascii().map(|i| i.is_ascii_alphabetic() || i == b'_').unwrap_or(false) => () }
                    .repeated(),
            )
            .to_slice()
    }

    /// Like [`ident`], but only accepts a specific identifier while rejecting trailing identifier characters.
    ///
    /// The output type of this parser is `I::Slice` (i.e: [`&str`] when `I` is [`&str`], and [`&[u8]`]
    /// when `I::Slice` is [`&[u8]`]).
    ///
    /// # Examples
    ///
    /// ```
    /// # use chumsky::prelude::*;
    /// let def = text::ascii::keyword::<_, _, extra::Err<Simple<char>>>("def");
    ///
    /// // Exactly 'def' was found
    /// assert_eq!(def.parse("def").into_result(), Ok("def"));
    /// // Exactly 'def' was found, with non-identifier trailing characters
    /// // This works because we made the parser lazy: it parses 'def' and ignores the rest
    /// assert_eq!(def.clone().lazy().parse("def(foo, bar)").into_result(), Ok("def"));
    /// // 'def' was found, but only as part of a larger identifier, so this fails to parse
    /// assert!(def.lazy().parse("define").has_errors());
    /// ```
    #[track_caller]
    pub fn keyword<'a, I, S, E>(keyword: S) -> impl Parser<'a, I, I::Slice, E> + Clone + 'a
    where
        I: StrInput<'a>,
        I::Slice: PartialEq,
        I::Token: fmt::Debug + 'a,
        S: Borrow<I::Slice> + Clone + 'a,
        E: ParserExtra<'a, I> + 'a,
    {
        /*
        #[cfg(debug_assertions)]
        {
            let mut cs = keyword.seq_iter();
            if let Some(c) = cs.next() {
                let c = c.borrow().to_char();
                assert!(c.is_ascii_alphabetic() || c == '_', "The first character of a keyword must be ASCII alphabetic or an underscore, not {:?}", c);
            } else {
                panic!("Keyword must have at least one character");
            }
            for c in cs {
                let c = c.borrow().to_char();
                assert!(c.is_ascii_alphanumeric() || c == '_', "Trailing characters of a keyword must be ASCII alphanumeric or an underscore, not {:?}", c);
            }
        }
        */
        ident()
            .try_map(move |s: I::Slice, span| {
                if &s == keyword.borrow() {
                    Ok(())
                } else {
                    Err(Error::expected_found(None, None, span))
                }
            })
            .to_slice()
    }
}

// Unicode is the default
pub use unicode::*;

/// Parsers and utilities for working with unicode inputs.
pub mod unicode {
    use super::*;

    use std::str::{Bytes, Chars};
    use unicode_segmentation::UnicodeSegmentation;

    /// A type containing one extended Unicode grapheme cluster.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Grapheme<'src> {
        inner: &'src str,
    }

    impl<'src> Grapheme<'src> {
        fn new(inner: &'src str) -> Self {
            Self { inner }
        }

        /// Creates a new grapheme with the character `'0'` inside it.
        pub fn digit_zero() -> Self {
            Self::new("0")
        }

        /// Gets an iterator over code points.
        pub fn code_points(self) -> Chars<'src> {
            self.inner.chars()
        }

        /// Gets an iterator over bytes.
        pub fn bytes(self) -> Bytes<'src> {
            self.inner.bytes()
        }

        /// Gets the slice of code points that are contained in the grapheme cluster.
        pub fn as_str(self) -> &'src str {
            self.inner
        }

        /// Gets the slice of bytes that are contained in the grapheme cluster.
        pub fn as_bytes(self) -> &'src [u8] {
            self.inner.as_bytes()
        }

        /// Splits the grapheme into the first code point and the remaining code points.
        pub fn split(self) -> (char, &'src str) {
            let mut iter = self.inner.chars();
            // The operation never falls because the grapheme always contains at least one code point.
            let first = iter.next().unwrap();
            (first, iter.as_str())
        }
    }

    /// A type containing any number of extended Unicode grapheme clusters.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Graphemes<'src> {
        inner: &'src str,
    }

    impl<'src> Graphemes<'src> {
        /// Create a new graphemes.
        pub fn new(inner: &'src str) -> Self {
            Self { inner }
        }

        /// Gets an iterator over graphemes.
        pub fn iter(self) -> GraphemesIter<'src> {
            self.into_iter()
        }

        /// Gets an iterator over code points.
        pub fn code_points(self) -> Chars<'src> {
            self.inner.chars()
        }

        /// Gets an iterator over bytes.
        pub fn bytes(self) -> Bytes<'src> {
            self.inner.bytes()
        }

        /// Gets the slice of code points that are contained in the string.
        pub fn as_str(self) -> &'src str {
            self.inner
        }

        /// Gets the slice of bytes that are contained in the string.
        pub fn as_bytes(self) -> &'src [u8] {
            self.inner.as_bytes()
        }
    }

    impl<'src> AsRef<str> for Graphemes<'src> {
        fn as_ref(&self) -> &str {
            self.as_str()
        }
    }

    impl<'src> AsRef<[u8]> for Graphemes<'src> {
        fn as_ref(&self) -> &[u8] {
            self.as_bytes()
        }
    }

    impl<'src> AsRef<Graphemes<'src>> for Graphemes<'src> {
        fn as_ref(&self) -> &Graphemes<'src> {
            self
        }
    }

    impl<'src> Borrow<str> for Graphemes<'src> {
        fn borrow(&self) -> &str {
            self.as_str()
        }
    }

    impl<'src> Borrow<[u8]> for Graphemes<'src> {
        fn borrow(&self) -> &[u8] {
            self.as_bytes()
        }
    }

    impl<'src> From<&'src str> for Graphemes<'src> {
        fn from(value: &'src str) -> Self {
            Graphemes::new(value)
        }
    }

    impl<'src> IntoIterator for Graphemes<'src> {
        type Item = Grapheme<'src>;

        type IntoIter = GraphemesIter<'src>;

        fn into_iter(self) -> Self::IntoIter {
            GraphemesIter::new(self)
        }
    }

    impl<'src> Input<'src> for Graphemes<'src> {
        type Cursor = usize;
        type Span = SimpleSpan<usize>;

        type Token = Grapheme<'src>;
        type MaybeToken = Grapheme<'src>;

        type Cache = Self;

        #[inline]
        fn begin(self) -> (Self::Cursor, Self::Cache) {
            (0, self)
        }

        #[inline]
        fn cursor_location(cursor: &Self::Cursor) -> usize {
            *cursor
        }

        #[inline(always)]
        unsafe fn next_maybe(
            this: &mut Self::Cache,
            cursor: &mut Self::Cursor,
        ) -> Option<Self::MaybeToken> {
            if *cursor < this.as_str().len() {
                // SAFETY: `cursor < self.len()` above guarantees cursor is in-bounds
                //         We only ever return cursors that are at a code point boundary.
                //         The `next()` implementation returns `None`, only in the
                //         situation of zero length of the remaining part of the string.
                //         And the Unicode standard guarantees that any sequence of code
                //         points is a valid sequence of grapheme clusters, so the
                //         behaviour of the `next()` function should not change.
                let c = this
                    .as_str()
                    .get_unchecked(*cursor..)
                    .graphemes(true)
                    .next()
                    .unwrap_unchecked();
                *cursor += c.len();
                Some(Grapheme::new(c))
            } else {
                None
            }
        }

        #[inline(always)]
        unsafe fn span(_this: &mut Self::Cache, range: Range<&Self::Cursor>) -> Self::Span {
            (*range.start..*range.end).into()
        }
    }

    impl<'src> ExactSizeInput<'src> for Graphemes<'src> {
        #[inline(always)]
        unsafe fn span_from(this: &mut Self::Cache, range: RangeFrom<&Self::Cursor>) -> Self::Span {
            (*range.start..this.as_str().len()).into()
        }
    }

    impl<'src> ValueInput<'src> for Graphemes<'src> {
        #[inline(always)]
        unsafe fn next(this: &mut Self::Cache, cursor: &mut Self::Cursor) -> Option<Self::Token> {
            Self::next_maybe(this, cursor)
        }
    }

    impl<'src> SliceInput<'src> for Graphemes<'src> {
        type Slice = Self;

        #[inline(always)]
        fn full_slice(this: &mut Self::Cache) -> Self::Slice {
            *this
        }

        #[inline(always)]
        unsafe fn slice(this: &mut Self::Cache, range: Range<&Self::Cursor>) -> Self::Slice {
            Graphemes::new(&this.as_str()[*range.start..*range.end])
        }

        #[inline(always)]
        unsafe fn slice_from(
            this: &mut Self::Cache,
            from: RangeFrom<&Self::Cursor>,
        ) -> Self::Slice {
            Graphemes::new(&this.as_str()[*from.start..])
        }
    }

    /// Grapheme iterator type.
    #[derive(Debug, Clone)]
    pub struct GraphemesIter<'src> {
        iter: unicode_segmentation::Graphemes<'src>,
    }

    impl<'src> GraphemesIter<'src> {
        /// Create a new grapheme iterator.
        pub fn new(graphemes: Graphemes<'src>) -> Self {
            Self {
                iter: graphemes.as_str().graphemes(true),
            }
        }

        /// Gets the slice of code points that are contained in the grapheme cluster.
        pub fn as_str(self) -> &'src str {
            self.iter.as_str()
        }
    }

    impl<'src> Iterator for GraphemesIter<'src> {
        type Item = Grapheme<'src>;

        #[inline]
        fn size_hint(&self) -> (usize, Option<usize>) {
            self.iter.size_hint()
        }

        #[inline]
        fn next(&mut self) -> Option<Self::Item> {
            self.iter.next().map(Grapheme::new)
        }
    }

    impl<'src> DoubleEndedIterator for GraphemesIter<'src> {
        #[inline]
        fn next_back(&mut self) -> Option<Self::Item> {
            self.iter.next_back().map(Grapheme::new)
        }
    }

    /// A parser that accepts an identifier.
    ///
    /// The output type of this parser is [`Char::Str`] (i.e: [`&str`] when `C` is [`char`], and [`&[u8]`] when `C` is
    /// [`u8`]).
    ///
    /// An identifier is defined as per "Default Identifiers" in [Unicode Standard Annex #31](https://www.unicode.org/reports/tr31/).
    #[must_use]
    pub fn ident<'a, I, E>() -> impl Parser<'a, I, I::Slice, E> + Copy
    where
        I: StrInput<'a, Token: 'a>,
        E: ParserExtra<'a, I>,
    {
        any()
            // Use try_map over filter to get a better error on failure
            .try_map(|c: I::Token, span| {
                if c.is_ident_start() {
                    Ok(c)
                } else {
                    Err(Error::expected_found([], Some(MaybeRef::Val(c)), span))
                }
            })
            .then(select! { c if (c as I::Token).is_ident_continue() => () }.repeated())
            .to_slice()
    }

    /// Like [`ident`], but only accepts a specific identifier while rejecting trailing identifier characters.
    ///
    /// The output type of this parser is `I::Slice` (i.e: [`&str`] when `I` is [`&str`], and [`&[u8]`]
    /// when `I::Slice` is [`&[u8]`]).
    ///
    /// # Examples
    ///
    /// ```
    /// # use chumsky::prelude::*;
    /// let def = text::ascii::keyword::<_, _, extra::Err<Simple<char>>>("def");
    ///
    /// // Exactly 'def' was found
    /// assert_eq!(def.parse("def").into_result(), Ok("def"));
    /// // Exactly 'def' was found, with non-identifier trailing characters
    /// // This works because we made the parser lazy: it parses 'def' and ignores the rest
    /// assert_eq!(def.clone().lazy().parse("def(foo, bar)").into_result(), Ok("def"));
    /// // 'def' was found, but only as part of a larger identifier, so this fails to parse
    /// assert!(def.lazy().parse("define").has_errors());
    /// ```
    #[track_caller]
    pub fn keyword<'a, I, S, E>(keyword: S) -> impl Parser<'a, I, I::Slice, E> + Clone + 'a
    where
        I: StrInput<'a>,
        I::Slice: PartialEq,
        I::Token: Char + fmt::Debug + 'a,
        S: Borrow<I::Slice> + Clone + 'a,
        E: ParserExtra<'a, I> + 'a,
    {
        /*
        #[cfg(debug_assertions)]
        {
            let mut cs = keyword.seq_iter();
            if let Some(c) = cs.next() {
                let c = c.borrow();
                assert!(
                    c.is_ident_start(),
                    "The first character of a keyword must be a valid unicode XID_START, not {:?}",
                    c
                );
            } else {
                panic!("Keyword must have at least one character");
            }
            for c in cs {
                let c = c.borrow();
                assert!(c.is_ident_continue(), "Trailing characters of a keyword must be valid as unicode XID_CONTINUE, not {:?}", c);
            }
        }
        */
        ident()
            .try_map(move |s: I::Slice, span| {
                if &s == keyword.borrow() {
                    Ok(())
                } else {
                    Err(Error::expected_found(None, None, span))
                }
            })
            .to_slice()
    }
}

// TODO: Better native form of semantic indentation that uses the context system?

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use std::fmt;

    fn make_ascii_kw_parser<'a, I>(s: I::Slice) -> impl Parser<'a, I, ()>
    where
        I: crate::StrInput<'a>,
        I::Slice: PartialEq + Clone,
        I::Token: fmt::Debug + 'a,
    {
        text::ascii::keyword(s).ignored()
    }

    fn make_unicode_kw_parser<'a, I>(s: I::Slice) -> impl Parser<'a, I, ()>
    where
        I: crate::StrInput<'a>,
        I::Slice: PartialEq + Clone,
        I::Token: fmt::Debug + 'a,
    {
        text::unicode::keyword(s).ignored()
    }

    fn test_ok<'a, P: Parser<'a, &'a str, &'a str>>(parser: P, input: &'a str) {
        assert_eq!(
            parser.parse(input),
            ParseResult {
                output: Some(input),
                errs: vec![]
            }
        );
    }

    fn test_err<'a, P: Parser<'a, &'a str, &'a str>>(parser: P, input: &'a str) {
        assert_eq!(
            parser.parse(input),
            ParseResult {
                output: None,
                errs: vec![EmptyErr::default()]
            }
        );
    }

    #[test]
    fn keyword_good() {
        make_ascii_kw_parser::<&str>("hello");
        make_ascii_kw_parser::<&str>("_42");
        make_ascii_kw_parser::<&str>("_42");

        make_unicode_kw_parser::<&str>("שלום");
        make_unicode_kw_parser::<&str>("привет");
        make_unicode_kw_parser::<&str>("你好");
    }

    #[test]
    fn ident() {
        let ident = text::ident::<&str, extra::Default>();
        test_ok(ident, "foo");
        test_ok(ident, "foo_bar");
        test_ok(ident, "foo_");
        test_ok(ident, "_foo");
        test_ok(ident, "_");
        test_ok(ident, "__");
        test_ok(ident, "__init__");
        test_err(ident, "");
        test_err(ident, ".");
        test_err(ident, "123");
    }

    /*
    #[test]
    #[should_panic]
    fn keyword_numeric() {
        make_ascii_kw_parser::<&str>("42");
    }

    #[test]
    #[should_panic]
    fn keyword_empty() {
        make_ascii_kw_parser::<&str>("");
    }

    #[test]
    #[should_panic]
    fn keyword_not_alphanum() {
        make_ascii_kw_parser::<&str>("hi\n");
    }

    #[test]
    #[should_panic]
    fn keyword_unicode_in_ascii() {
        make_ascii_kw_parser::<&str>("שלום");
    }
    */
}
