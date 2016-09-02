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
//! A symbol reader reads one symbol at a time from a source
//!

use std::slice::Iter;
use std::io::Read;
use std::io::Bytes;
use std::str::Chars;

///
/// A symbol reader reads one symbol at a time from a source
///
pub trait SymbolReader<Symbol> {
    ///
    /// Reads the next symbol from this reader
    ///
    fn next_symbol(&mut self) -> Option<Symbol>;
}

///
/// Trait that can be implemented by things that can construct a symbol reader
///
pub trait SymbolSource<'a, Symbol> {
    /// The type of symbol reader that will be generated by this call
    type SymbolReader: SymbolReader<Symbol>+'a;

    /// Returns a new object that can read the symbols from this one
    fn read_symbols(self) -> Self::SymbolReader;
}

impl<'a, Symbol: Clone+'a> SymbolSource<'a, Symbol> for &'a Vec<Symbol> {
    type SymbolReader = Iter<'a, Symbol>;

    fn read_symbols(self) -> Self::SymbolReader {
        self.iter()
    }
}

impl<'a, Symbol: Clone+'a> SymbolReader<Symbol> for Iter<'a, Symbol> {
    fn next_symbol(&mut self) -> Option<Symbol> {
        if let Some(sym) = self.next() {
            Some(sym.clone())
        } else {
            None
        }
    }
}

// Can read from streams

//
// TODO: implementing SymbolSource for Read would be nice
// It's just a matter of calling bytes() on the appropriate type, but Rust's type system is simultaneously too clever and not clever enough
// to allow that in any sensible way. You can't do it an an arbitrary Read because the Bytes object is not sized without knowing its real
// type; I couldn't figure out a trick using the Self type to get around that, though those do work sometimes. 
// `impl<X: Read> SymbolReader for X` sounds like a nice idea but Rust thinks X can be anything and won't allow any other implementations
// of the interface, which makes that useless too.
// Screwing around with lifetimes and references like below seems like it should work, but Rust doesn't seem to see the implementations on
// actual streams.
//

/*
impl<'a> SymbolSource<'a, u8> for &'a mut Read {
    type SymbolReader = ByteSymbolReader<Self>;

    fn read_symbols(self) -> Self::SymbolReader {
        ByteSymbolReader::new(self.bytes())
    }
}

impl<'a> SymbolSource<'a, u8> for &'a mut BufRead {
    type SymbolReader = ByteSymbolReader<Self>;

    fn read_symbols(self) -> Self::SymbolReader {
        ByteSymbolReader::new(self.bytes())
    }
}
*/

///
/// The ByteSymbolReader turns a `std::io::Bytes` object into a symbol reader
///
pub struct ByteSymbolReader<Reader: Read> {
    bytes: Bytes<Reader>
}

impl<Reader: Read> ByteSymbolReader<Reader> {
    pub fn new(bytes: Bytes<Reader>) -> ByteSymbolReader<Reader> {
        ByteSymbolReader { bytes: bytes }
    }

    pub fn from(reader: Reader) -> ByteSymbolReader<Reader> {
        Self::new(reader.bytes())
    }
}

impl<Reader: Read> SymbolReader<u8> for ByteSymbolReader<Reader> {
    fn next_symbol(&mut self) -> Option<u8> {
        if let Some(Ok(sym)) = self.bytes.next() {
            Some(sym.clone())
        } else {
            None
        }
    }
}

//
// Can read from strings 
//
impl<'a> SymbolSource<'a, char> for &'a str {
    type SymbolReader = Chars<'a>;

    fn read_symbols(self) -> Self::SymbolReader {
        self.chars()
    }
}

impl<'a> SymbolReader<char> for Chars<'a> {
    fn next_symbol(&mut self) -> Option<char> {
        self.next()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn can_read_from_vec() {
        let source      = vec![1, 2, 3];
        let mut reader  = source.read_symbols();

        assert!(reader.next_symbol() == Some(1));
        assert!(reader.next_symbol() == Some(2));
        assert!(reader.next_symbol() == Some(3));
        assert!(reader.next_symbol() == None);
    }

    #[test]
    fn can_read_from_bytes_reader() {
        let array: [u8; 3] = [1, 2, 3];
        let slice = &array[..];
        let mut reader = ByteSymbolReader::from(slice);

        assert!(reader.next_symbol() == Some(1));
        assert!(reader.next_symbol() == Some(2));
        assert!(reader.next_symbol() == Some(3));
        assert!(reader.next_symbol() == None);
    }

    #[test]
    fn can_read_from_string_reader() {
        let mut reader = "abc".read_symbols();

        assert!(reader.next_symbol() == Some('a'));
        assert!(reader.next_symbol() == Some('b'));
        assert!(reader.next_symbol() == Some('c'));
        assert!(reader.next_symbol() == None);
    }
}
