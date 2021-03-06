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
//! A library for working with deterministic and non-deterministic finite-state automata.
//!

#[macro_use] extern crate serde;
#[macro_use] extern crate serde_derive;

pub use self::countable::*;
pub use self::symbol_range::*;
pub use self::symbol_reader::*;
pub use self::state_machine::*;
pub use self::pattern_matcher::*;
pub use self::ndfa::*;
pub use self::regular_pattern::*;
pub use self::regular_expression::*;
pub use self::dfa_builder::*;
pub use self::symbol_range_dfa::*;
pub use self::dfa_compiler::*;
pub use self::prepare::*;
pub use self::matches::*;
pub use self::tape::*;
pub use self::split_reader::*;
pub use self::tokenizer::*;
pub use self::tagged_stream::*;

pub mod countable;
pub mod symbol_range;
pub mod symbol_reader;
pub mod state_machine;
pub mod overlapping_symbols;
pub mod pattern_matcher;
pub mod ndfa;
pub mod regular_pattern;
pub mod regular_expression;
pub mod dfa_builder;
pub mod symbol_range_dfa;
pub mod dfa_compiler;
pub mod prepare;
pub mod matches;
pub mod tape;
pub mod split_reader;
pub mod tokenizer;
pub mod tagged_stream;
