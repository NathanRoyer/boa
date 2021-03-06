mod catch;
mod finally;

#[cfg(test)]
mod tests;

use self::catch::Catch;
use self::finally::Finally;
use super::block::Block;
use crate::syntax::lexer::TokenKind;
use crate::{
    syntax::{
        ast::{node::Try, Keyword},
        parser::{AllowAwait, AllowReturn, AllowYield, Cursor, ParseError, TokenParser},
    },
    BoaProfiler,
};

use std::io::Read;

/// Try...catch statement parsing
///
/// More information:
///  - [MDN documentation][mdn]
///  - [ECMAScript specification][spec]
///
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/try...catch
/// [spec]: https://tc39.es/ecma262/#sec-try-statement
#[derive(Debug, Clone, Copy)]
pub(super) struct TryStatement {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
    allow_return: AllowReturn,
}

impl TryStatement {
    /// Creates a new `TryStatement` parser.
    pub(super) fn new<Y, A, R>(allow_yield: Y, allow_await: A, allow_return: R) -> Self
    where
        Y: Into<AllowYield>,
        A: Into<AllowAwait>,
        R: Into<AllowReturn>,
    {
        Self {
            allow_yield: allow_yield.into(),
            allow_await: allow_await.into(),
            allow_return: allow_return.into(),
        }
    }
}

impl<R> TokenParser<R> for TryStatement
where
    R: Read,
{
    type Output = Try;

    fn parse(self, cursor: &mut Cursor<R>) -> Result<Try, ParseError> {
        let _timer = BoaProfiler::global().start_event("TryStatement", "Parsing");
        // TRY
        cursor.expect(Keyword::Try, "try statement")?;

        let try_clause =
            Block::new(self.allow_yield, self.allow_await, self.allow_return).parse(cursor)?;

        let next_token = cursor.peek(0)?.ok_or(ParseError::AbruptEnd)?;

        if next_token.kind() != &TokenKind::Keyword(Keyword::Catch)
            && next_token.kind() != &TokenKind::Keyword(Keyword::Finally)
        {
            return Err(ParseError::expected(
                vec![
                    TokenKind::Keyword(Keyword::Catch),
                    TokenKind::Keyword(Keyword::Finally),
                ],
                next_token.clone(),
                "try statement",
            ));
        }

        let catch = if next_token.kind() == &TokenKind::Keyword(Keyword::Catch) {
            Some(Catch::new(self.allow_yield, self.allow_await, self.allow_return).parse(cursor)?)
        } else {
            None
        };

        let next_token = cursor.peek(0)?;
        let finally_block = if let Some(token) = next_token {
            match token.kind() {
                TokenKind::Keyword(Keyword::Finally) => Some(
                    Finally::new(self.allow_yield, self.allow_await, self.allow_return)
                        .parse(cursor)?,
                ),
                _ => None,
            }
        } else {
            None
        };

        Ok(Try::new(try_clause, catch, finally_block))
    }
}
