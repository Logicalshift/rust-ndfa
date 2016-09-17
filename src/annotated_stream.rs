//
//   Copyright 2016 Andrew Hunter
//
//   Licensed under the Apache License, Version 2.0 (the "License");
//   you may not use this file except in compliance with the License.
//   You may obtain a copy of the License at
//
//       http://www.apache.org/licenses/LICENSE-2.0
//
//   Unless required by applicable law or agreed to in writing, software
//   distributed under the License is distributed on an "AS IS" BASIS,
//   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//   See the License for the specific language governing permissions and
//   limitations under the License.
//

//!
//! An annotated stream represents the result of adding lexical symbols to an input stream.
//!
//! This is useful when an input stream is turned into a series of tokens, but the original values of those tokens still need to
//! be retrieved. For example, in a hypothetical programming language we might tokenize the string `a+1` as `<identifier> '+' <number>`.
//! That's useful for passing into a parser which can recognise it as an expression. However, it's no good for evaluating the
//! expression as for that it's necessary to know that the first identifier is `a`.
//!
//! An annotated stream stores both the input and the output to the tokenizer, which makes it possible to retrieve the input that
//! was matched to produce each token.
//!
//! These streams can be made by manually annotating some input symbols:
//!
//! ```
//! # use concordance::*;
//! #[derive(Ord, PartialOrd, Eq, PartialEq, Clone)]
//! enum LexToken {
//!     Identifier, Number, Plus
//! };
//!
//! let mut annotator = Annotator::new();
//! 
//! annotator.append_input(vec!['1', '2']);
//! annotator.token(LexToken::Number);
//!
//! let annotated_stream = annotator.finish();          // == Number(12)
//! # assert!(annotated_stream.read_output().to_vec().len() == 1);
//! ```
//!
//! It can also be created by applying a tokenizer to an input stream:
//!
//! ```
//! # use concordance::*;
//! #[derive(Ord, PartialOrd, Eq, PartialEq, Clone)]
//! enum LexToken {
//!     Identifier, Number, Plus
//! };
//!
//! let mut token_matcher = TokenMatcher::new();
//! token_matcher.add_pattern((MatchRange('a', 'z').or(MatchRange('A', 'Z'))).repeat_forever(1), LexToken::Identifier);
//! token_matcher.add_pattern(MatchRange('0', '9').repeat_forever(1), LexToken::Number);
//! token_matcher.add_pattern("+".into_pattern(), LexToken::Plus);
//!
//! let tokenizer = token_matcher.prepare_to_match();
//!
//! let annotated_stream = AnnotatedStream::from_tokenizer(&tokenizer, "a+1".read_symbols());
//! # assert!(annotated_stream.read_output().to_vec().len() == 3);
//! ```
//!
//! The input and the output can be read independently:
//!
//! ```
//! # use concordance::*;
//! # #[derive(Ord, PartialOrd, Eq, PartialEq, Clone)]
//! # enum LexToken {
//! #     Identifier, Number, Plus
//! # };
//! # 
//! # let mut token_matcher = TokenMatcher::new();
//! # token_matcher.add_pattern((MatchRange('a', 'z').or(MatchRange('A', 'Z'))).repeat_forever(1), LexToken::Identifier);
//! # token_matcher.add_pattern(MatchRange('0', '9').repeat_forever(1), LexToken::Number);
//! # token_matcher.add_pattern("+".into_pattern(), LexToken::Plus);
//! # 
//! # let tokenizer = token_matcher.prepare_to_match();
//! # 
//! # let annotated_stream = AnnotatedStream::from_tokenizer(&tokenizer, "a+1".read_symbols());
//! let mut input_stream  = annotated_stream.read_input();  // == ['a' '+' '1']
//! let mut output_stream = annotated_stream.read_output(); // == [Identifier Plus Number]
//! # assert!(input_stream.to_vec() == vec!['a', '+', '1']);
//! ```
//!
//! It's possible to work out which token each input symbol corresponds to:
//!
//! ```
//! # use concordance::*;
//! # #[derive(Ord, PartialOrd, Eq, PartialEq, Clone)]
//! # enum LexToken {
//! #     Identifier, Number, Plus
//! # };
//! # 
//! # let mut token_matcher = TokenMatcher::new();
//! # token_matcher.add_pattern((MatchRange('a', 'z').or(MatchRange('A', 'Z'))).repeat_forever(1), LexToken::Identifier);
//! # token_matcher.add_pattern(MatchRange('0', '9').repeat_forever(1), LexToken::Number);
//! # token_matcher.add_pattern("+".into_pattern(), LexToken::Plus);
//! # 
//! # let tokenizer = token_matcher.prepare_to_match();
//! # 
//! # let annotated_stream = AnnotatedStream::from_tokenizer(&tokenizer, "a+1".read_symbols());
//! let middle_token = annotated_stream.find_token(1); // == Some(Token { matched: &['+'], output: Plus })
//! # assert!(middle_token.is_some());
//! # let unwrapped = middle_token.unwrap();
//! # assert!(annotated_stream.input_for_token(&unwrapped) == &['+']);
//! # assert!(unwrapped.output == LexToken::Plus);
//! ```
//!
//! And it's possible to read the fully tokenized form of the stream, or part of the stream:
//!
//! ```
//! # use concordance::*;
//! # #[derive(Ord, PartialOrd, Eq, PartialEq, Clone)]
//! # enum LexToken {
//! #     Identifier, Number, Plus
//! # };
//! # 
//! # let mut token_matcher = TokenMatcher::new();
//! # token_matcher.add_pattern((MatchRange('a', 'z').or(MatchRange('A', 'Z'))).repeat_forever(1), LexToken::Identifier);
//! # token_matcher.add_pattern(MatchRange('0', '9').repeat_forever(1), LexToken::Number);
//! # token_matcher.add_pattern("+".into_pattern(), LexToken::Plus);
//! # 
//! # let tokenizer = token_matcher.prepare_to_match();
//! # 
//! # let annotated_stream = AnnotatedStream::from_tokenizer(&tokenizer, "a+1".read_symbols());
//! let tokens      = annotated_stream.read_tokens();              // == stream containing Token objects representing 'Identifier Plus Number'
//! let more_tokens = annotated_stream.read_tokens_in_range(1..3); // == stream containing Token objects representing just 'Plus Number'
//! ```
//!

use std::ops::Range;

use super::countable::*;
use super::tokenizer::*;
use super::symbol_reader::*;
use super::symbol_range_dfa::*;

///
/// An annotated stream represents an original stream, with ranges tagged with tokens. This can be used to map between a
/// tokenized stream and the original characters.
///
#[derive(Clone)]
pub struct AnnotatedStream<InputType, TokenType> {
    /// The original stream that was tokenized
    original: Vec<InputType>,

    /// The tokenized version and where in the original they appear, in order
    tokenized: Vec<(TokenType, Range<usize>)>
}

impl<InputSymbol: Clone+Ord+Countable, OutputSymbol: Clone+Ord+'static> AnnotatedStream<InputSymbol, OutputSymbol> {
    ///
    /// Given a stream and a DFA, tokenizes the stream and annotates it with the appropriate tokens
    ///
    pub fn from_tokenizer<Reader: SymbolReader<InputSymbol>>(dfa: &SymbolRangeDfa<InputSymbol, OutputSymbol>, reader: Reader) -> AnnotatedStream<InputSymbol, OutputSymbol> {
        // Capture the contents of the original reader (we store them in the result)
        let mut reader_m = reader;
        let     original = reader_m.to_vec();
        let mut tokens   = vec![];

        {
            // Create a new reader to read our captured symbols
            let token_reader  = original.read_symbols();
            let mut tokenizer = Tokenizer::new_prepared(token_reader, dfa);

            // Start tokenizing
            let mut pos: usize = 0;

            loop {
                // Tokenize the next symbol
                let next_token  = tokenizer.next_symbol();
                let final_pos   = tokenizer.get_source_position();

                if let Some(output) = next_token {
                    // Got a token
                    tokens.push((output, pos..final_pos));

                    // Next token begins after this one
                    pos = final_pos;
                } else if !tokenizer.at_end_of_reader() {
                    // Skip tokens that don't form a match (returned none + not at the end of the reader)
                    // Try again in case there are further tokens

                    // TODO: it might actually be better to do this in the pattern matcher itself rather than here.
                    //  Need to devise a way to make an NDFA that can match something along with the 'longest unmatched' string to do this
                    pos += 1;
                    tokenizer.skip_input();
                } else {
                    // Reached the end of the input
                    break;
                }
            }
        }

        // Annotated stream is ready
        AnnotatedStream { original: original, tokenized: tokens }
    }

    ///
    /// Reads the original stream for the original input for this stream
    ///
    pub fn read_input<'a>(&'a self) -> Box<SymbolReader<InputSymbol> + 'a> {
        Box::new(self.original.read_symbols())
    }

    ///
    /// Retrieves the input symbols for a particular token
    ///
    #[inline]
    pub fn input_for_token<'a>(&'a self, token: &Token<OutputSymbol>) -> &'a [InputSymbol] {
        &self.input_for_range(token.location.clone())
    }

    ///
    /// Retrieves the input symbols for a particular range
    ///
    #[inline]
    pub fn input_for_range<'a>(&'a self, range: Range<usize>) -> &'a [InputSymbol] {
        &self.original[range]
    }

    ///
    /// Retrieves the number of tokens in the input to this stream
    ///
    pub fn input_len(&self) -> usize {
        self.original.len()
    }

    ///
    /// Retrieves the number of tokens in the output
    ///
    pub fn output_len(&self) -> usize {
        self.tokenized.len()
    }

    ///
    /// Reads the tokenized output stream (only for the symbols that were recognised)
    ///
    pub fn read_output<'a>(&'a self) -> Box<SymbolReader<OutputSymbol> + 'a> {
        let full_output  = self.tokenized.read_symbols();
        let only_symbols = full_output.map_symbols(|(token, _)| token); 

        Box::new(only_symbols)
    }

    ///
    /// Reads the annotated stream as a series of tokens
    ///
    pub fn read_tokens<'a>(&'a self) -> Box<SymbolReader<Token<OutputSymbol>> + 'a> {
        let full_output = self.tokenized.read_symbols();
        let with_tokens = full_output.map_symbols(move |(output, location)| {
            Token {
                location: location.clone(),
                output:   output
            }
        });

        Box::new(with_tokens)
    }

    ///
    /// Finds the index into the tokenized list of the token corresponding to the specified position
    ///
    /// Returns Err(index_after) if there's no range corresponding to the position in this stream
    ///
    fn find_token_index(&self, position: usize) -> Result<usize, usize> {
        // Try to find the specified position: assumes the tokens are in order (which they are if we generated this stream from left to right)
        let found = self.tokenized.binary_search_by(|&(_, ref location)| location.start.cmp(&position));
        
        // We're only searching on the start position: if we don't find it exactly, then the token might be in the preceding index
        match found {
            Ok(index) => Ok(index),

            Err(index) => {
                if index == 0 {
                    Err(0)
                } else {
                    if self.tokenized[index-1].1.end > position {
                        Ok(index-1)
                    } else {
                        Err(index)
                    }
                }
            }
        }
    }

    ///
    /// Finds the token at the specified position in this stream
    ///
    pub fn find_token<'a>(&'a self, position: usize) -> Option<Token<OutputSymbol>> {
        let found_index = self.find_token_index(position).ok();

        // Build a token for this location
        found_index.map(move |index| {
            let (ref output, ref location) = self.tokenized[index];

            Token { 
                location: location.clone(),
                output:   output.clone(),
            }
        })
    }

    ///
    /// Returns a symbol reader that reads all the tokens in a particular range of source positions
    ///
    pub fn read_tokens_in_range<'a>(&'a self, search_location: Range<usize>) -> Box<SymbolReader<Token<OutputSymbol>> + 'a> {
        // Find the initial index
        let start_index = match self.find_token_index(search_location.start) { Ok(index) => index, Err(index) => index };

        // Find the final index
        let mut end_index = start_index;
        loop {
            // Don't go over the end of the array
            if end_index >= self.tokenized.len() {
                break;
            }

            // Stop if we find a value after the range we're search for
            let (_, ref token_location) = self.tokenized[end_index];
            if token_location.start >= search_location.end {
                break;
            }

            end_index += 1;
        }

        // Generate a symbol reader for this range
        let token_range = &self.tokenized[start_index..end_index];
        let with_tokens = token_range.iter().map_symbols(move |(output, location)| {
            Token { 
                location: location.clone(),
                output:   output.clone(),
            }
        });

        Box::new(with_tokens)
    }
}

///
/// A token represents an individual item in an annotated stream
///
#[derive(Eq, PartialEq, Clone)]
pub struct Token<OutputSymbol> {
    /// Where this token appears in the output
    pub location: Range<usize>,

    /// The output symbol that was matched for this token
    pub output: OutputSymbol
}

impl<OutputSymbol> Token<OutputSymbol> {
    ///
    /// Creates a new token
    ///
    pub fn new(location: Range<usize>, output: OutputSymbol) -> Token<OutputSymbol> {
        Token { location: location, output: output }
    }
}

///
/// An annotator makes it possible to create an annotated stream by manually tagging an input stream
///
pub struct Annotator<InputSymbol, TokenType> {
    /// The stream that is being built
    stream: AnnotatedStream<InputSymbol, TokenType>,

    /// The start position of the current output symbol
    start_pos: usize
}

impl<InputSymbol, TokenType> Annotator<InputSymbol, TokenType> {
    ///
    /// Creates a new annotator
    ///
    pub fn new() -> Annotator<InputSymbol, TokenType> {
        Annotator { stream: AnnotatedStream { original: vec![], tokenized: vec![] }, start_pos: 0 }
    }

    ///
    /// Adds a new input symbol
    ///
    pub fn push_input(&mut self, symbol: InputSymbol) {
        self.stream.original.push(symbol);
    }

    ///
    /// Appends a vector of input symbols to the result
    ///
    pub fn append_input(&mut self, mut input: Vec<InputSymbol>) {
        self.stream.original.append(&mut input);
    }

    ///
    /// Annotates the symbols since the last token with the specified token
    ///
    pub fn token(&mut self, token: TokenType) {
        let pos = self.stream.original.len();

        self.stream.tokenized.push((token, self.start_pos..pos));
        self.start_pos = pos;
    }

    ///
    /// Skips the symbols (leaving them without a token) since the last token
    ///
    pub fn skip(&mut self) {
        self.start_pos = self.stream.original.len();
    }

    ///
    /// Finishes the annotated stream and returns the result
    ///
    pub fn finish(self) -> AnnotatedStream<InputSymbol, TokenType> {
        self.stream
    }
}

#[cfg(test)]
mod test {
    use std::iter::FromIterator;
    pub use super::super::*;

    #[test]
    fn can_annotate_stream() {
        #[derive(Ord, PartialOrd, Eq, PartialEq, Clone)]
        enum TestToken {
            Digit,
            Whitespace
        }

        let mut token_matcher = TokenMatcher::new();
        token_matcher.add_pattern(MatchRange('0', '9').repeat_forever(0), TestToken::Digit);
        token_matcher.add_pattern(" ".repeat_forever(0), TestToken::Whitespace);

        let dfa   = token_matcher.prepare_to_match();
        let input = "12 42 13";

        let annotated = AnnotatedStream::from_tokenizer(&dfa, input.read_symbols());

        let annotated_input  = annotated.read_input().to_vec();
        let annotated_output = annotated.read_output().to_vec();
        let annotated_tokens = annotated.read_tokens().to_vec();

        assert!(annotated_input == vec!['1', '2', ' ', '4', '2', ' ', '1', '3']);
        assert!(annotated_output == vec![TestToken::Digit, TestToken::Whitespace, TestToken::Digit, TestToken::Whitespace, TestToken::Digit]);

        assert!(annotated_tokens[0].location.start == 0);
        assert!(annotated_tokens[0].location.end == 2);
        assert!(annotated_tokens[0].output == TestToken::Digit);
    }

    #[test]
    fn can_find_token() {
        #[derive(Ord, PartialOrd, Eq, PartialEq, Clone)]
        enum TestToken {
            Digit,
            Whitespace
        }

        let mut token_matcher = TokenMatcher::new();
        token_matcher.add_pattern(MatchRange('0', '9').repeat_forever(0), TestToken::Digit);
        token_matcher.add_pattern(" ".repeat_forever(0), TestToken::Whitespace);

        let dfa   = token_matcher.prepare_to_match();
        let input = "12 42 13";

        let annotated  = AnnotatedStream::from_tokenizer(&dfa, input.read_symbols());

        let fortytwo   = annotated.find_token(4).expect("No token");
        let whitespace = annotated.find_token(5).expect("No token");

        assert!(fortytwo.location.start == 3);
        assert!(fortytwo.location.end == 5);
        assert!(fortytwo.output == TestToken::Digit);
        assert!(annotated.input_for_token(&fortytwo) == &['4', '2']);

        assert!(whitespace.location.start == 5);
        assert!(whitespace.location.end == 6);
        assert!(whitespace.output == TestToken::Whitespace);
        assert!(annotated.input_for_token(&whitespace) == &[' ']);
    }

    #[test]
    fn can_read_token_range() {
        #[derive(Ord, PartialOrd, Eq, PartialEq, Clone)]
        enum TestToken {
            Digit,
            Whitespace
        }

        let mut token_matcher = TokenMatcher::new();
        token_matcher.add_pattern(MatchRange('0', '9').repeat_forever(0), TestToken::Digit);
        token_matcher.add_pattern(" ".repeat_forever(0), TestToken::Whitespace);

        let dfa   = token_matcher.prepare_to_match();
        let input = "12 42 13";

        let annotated = AnnotatedStream::from_tokenizer(&dfa, input.read_symbols());

        let some_output = annotated.read_tokens_in_range(4..7).to_vec();
        let input_strings = some_output.iter().map(|token| {
            let res: String = annotated.input_for_token(&token).iter().cloned().collect();
            res
        });
        let input_strings_vec = Vec::from_iter(input_strings);

        assert!(input_strings_vec == vec!["42", " ", "13"]);

        let different_output = annotated.read_tokens_in_range(0..4).to_vec();
        let different_strings = different_output.iter().map(|token| {
            let res: String = annotated.input_for_token(&token).iter().cloned().collect();
            res
        });
        let different_strings_vec = Vec::from_iter(different_strings);

        assert!(different_strings_vec == vec!["12", " ", "42"]);
    }

    #[test]
    fn can_get_input_length() {
        #[derive(Ord, PartialOrd, Eq, PartialEq, Clone)]
        enum TestToken {
            Digit,
            Whitespace
        }

        let mut token_matcher = TokenMatcher::new();
        token_matcher.add_pattern(MatchRange('0', '9').repeat_forever(0), TestToken::Digit);
        token_matcher.add_pattern(" ".repeat_forever(0), TestToken::Whitespace);

        let dfa   = token_matcher.prepare_to_match();
        let input = "12 42 13";

        let annotated = AnnotatedStream::from_tokenizer(&dfa, input.read_symbols());

        assert!(annotated.input_len() == 8);
    }

    #[test]
    fn can_get_output_length() {
        #[derive(Ord, PartialOrd, Eq, PartialEq, Clone)]
        enum TestToken {
            Digit,
            Whitespace
        }

        let mut token_matcher = TokenMatcher::new();
        token_matcher.add_pattern(MatchRange('0', '9').repeat_forever(0), TestToken::Digit);
        token_matcher.add_pattern(" ".repeat_forever(0), TestToken::Whitespace);

        let dfa   = token_matcher.prepare_to_match();
        let input = "12 42 13";

        let annotated = AnnotatedStream::from_tokenizer(&dfa, input.read_symbols());

        assert!(annotated.output_len() == 5);
    }

    #[test]
    fn can_get_input_range() {
        #[derive(Ord, PartialOrd, Eq, PartialEq, Clone)]
        enum TestToken {
            Digit,
            Whitespace
        }

        let mut token_matcher = TokenMatcher::new();
        token_matcher.add_pattern(MatchRange('0', '9').repeat_forever(0), TestToken::Digit);
        token_matcher.add_pattern(" ".repeat_forever(0), TestToken::Whitespace);

        let dfa   = token_matcher.prepare_to_match();
        let input = "12 42 13";

        let annotated = AnnotatedStream::from_tokenizer(&dfa, input.read_symbols());

        assert!(annotated.input_for_range(3..6) == &['4', '2', ' ']);
    }

    #[test]
    fn can_annotate_manually() {
        #[derive(Ord, PartialOrd, Eq, PartialEq, Clone)]
        enum TestToken {
            Digit,
            Whitespace
        }

        let mut annotator = Annotator::new();

        annotator.push_input('1');
        annotator.push_input('2');
        annotator.token(TestToken::Digit);

        annotator.push_input(' ');
        annotator.token(TestToken::Whitespace);

        annotator.push_input('4');
        annotator.push_input('2');
        annotator.token(TestToken::Digit);

        annotator.push_input(' ');
        annotator.token(TestToken::Whitespace);

        annotator.append_input(vec!['1', '3']);
        annotator.token(TestToken::Digit);

        let annotated = annotator.finish();

        assert!(annotated.input_for_range(3..6) == &['4', '2', ' ']);
        assert!(annotated.output_len() == 5);
        assert!(annotated.input_len() == 8);

        let fortytwo   = annotated.find_token(4).expect("No token");
        let whitespace = annotated.find_token(5).expect("No token");

        assert!(fortytwo.location.start == 3);
        assert!(fortytwo.location.end == 5);
        assert!(fortytwo.output == TestToken::Digit);
        assert!(annotated.input_for_token(&fortytwo) == &['4', '2']);

        assert!(whitespace.location.start == 5);
        assert!(whitespace.location.end == 6);
        assert!(whitespace.output == TestToken::Whitespace);
        assert!(annotated.input_for_token(&whitespace) == &[' ']);
    }
}
