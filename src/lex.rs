use token::{LexResult, Token, TokenTrait};

pub struct Lexer<'a> {
    unlexed: &'a str,
}

pub trait LexerTrait<'a> {
    fn get_unlexed(&self) -> &'a str;
    fn lex(&self) -> Option<Vec<Token>>;
}

impl<'a> LexerTrait<'a> for Lexer<'a> {
    fn get_unlexed(&self) -> &'a str {
        self.unlexed
    }

    fn lex(&self) -> Option<Vec<Token>> {
        let mut toks = Vec::new();
        for unlexed_char in self.unlexed.to_string().chars() {
            match toks.pop() {
                None => toks.push(Token::new(unlexed_char)),
                Some(mut tok) => {
                    if tok.get_status() {
                        toks.push(tok);
                        toks.push(Token::new(unlexed_char));
                    } else {
                        match tok.lex_char(unlexed_char) {
                            LexResult::Pass => toks.push(tok),
                            LexResult::Finish => toks.push(tok),
                            LexResult::FinishNew => {
                                toks.push(tok);
                                toks.push(Token::new(unlexed_char));
                            },
                            LexResult::FinishDelete => toks.push(tok),
                            LexResult::Continue => toks.push(tok),
                            LexResult::Fail => return None
                        }
                    }
                }
            }
        }
        Some(toks)
    }
}

impl<'a> Lexer<'a> {
    pub fn new(newunlexed: &'a str) -> Self {
        Lexer {
            unlexed: newunlexed
        }
    }
}
