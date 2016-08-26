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

use std::rc::*;

///
/// Identifies a state in a state machine
///
pub type StateId = u32;

///
/// Trait implemented by objects that represent a state machine, whose transitions depend on a particular symbol type
///
/// These state machines can be nondeterministic - which is to say, have more than one transition per state. They always
/// start in state 0.
///
pub trait StateMachine<InputSymbol, OutputSymbol> {
    ///
    /// Returns the number of states in this state machine
    ///
    fn count_states(&self) -> StateId;

    ///
    /// Returns the transitions for a particular symbol 
    ///
    fn get_transitions_for_state(&self, state: StateId) -> Vec<(InputSymbol, StateId)>;

    ///
    /// If a state is an accepting state, then this returns the output symbol that should be produced if this is the longest match
    ///
    fn output_symbol_for_state(&self, state: StateId) -> Option<&OutputSymbol>;
}

///
/// Trait implemented by state machines that can be altered
///
pub trait MutableStateMachine<InputSymbol, OutputSymbol> : StateMachine<InputSymbol, OutputSymbol> {
    ///
    /// Adds a transition from a particular state to another on seeing a symbol
    ///
    fn add_transition(&mut self, state: StateId, for_symbol: InputSymbol, new_state: StateId);

    ///
    /// Sets the output symbol to use for a particular state
    ///
    fn set_output_symbol(&mut self, state: StateId, new_output_symbol: OutputSymbol);
}

///
/// Trait used to indicate that a particular state machine is deterministic (has at most one 
/// transition per symbol from the original)
///
pub trait DeterministicStateMachine<InputSymbol, OutputSymbol> : StateMachine<InputSymbol, OutputSymbol> { }

///
/// Any reference to a state machine is also a state machine
///
impl<InputSymbol, OutputSymbol> StateMachine<InputSymbol, OutputSymbol> for Rc<StateMachine<InputSymbol, OutputSymbol>> {
    #[inline]
    fn count_states(&self) -> StateId {
        (**self).count_states()
    }

    #[inline]
    fn get_transitions_for_state(&self, state: StateId) -> Vec<(InputSymbol, StateId)> {
        (**self).get_transitions_for_state(state)
    }

    #[inline]
    fn output_symbol_for_state(&self, state: StateId) -> Option<&OutputSymbol> {
        (**self).output_symbol_for_state(state)
    }
}

impl<InputSymbol, OutputSymbol> StateMachine<InputSymbol, OutputSymbol> for Rc<DeterministicStateMachine<InputSymbol, OutputSymbol>> {
    #[inline]
    fn count_states(&self) -> StateId {
        (**self).count_states()
    }

    #[inline]
    fn get_transitions_for_state(&self, state: StateId) -> Vec<(InputSymbol, StateId)> {
        (**self).get_transitions_for_state(state)
    }

    #[inline]
    fn output_symbol_for_state(&self, state: StateId) -> Option<&OutputSymbol> {
        (**self).output_symbol_for_state(state)
    }
}

impl<InputSymbol, OutputSymbol> DeterministicStateMachine<InputSymbol, OutputSymbol> for Rc<DeterministicStateMachine<InputSymbol, OutputSymbol>> {
}
