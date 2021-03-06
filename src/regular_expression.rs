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
//! Regular expressions are a way to expression patterns in a regular language. They're only useful for character streams.
//!

use super::regular_pattern::*;

impl Pattern<char> {
    ///
    /// Creates a new pattern from a regular expression
    ///
    pub fn from_regex(pattern: &str) -> Pattern<char> {
        unimplemented!()
    }
}