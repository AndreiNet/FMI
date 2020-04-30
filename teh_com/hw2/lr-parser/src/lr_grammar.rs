pub mod types;
pub mod helpers;
pub mod states;

use std::collections::BTreeMap;

use types::*;
use super::grammar::types::*;

pub struct LRGrammar<S: LRState> {
    grammar: Grammar,
    states: Vec<S>,
    init_state: S,
    action: BTreeMap<(S, Term), Move<S>>,
}

impl GrammarInfo for Grammar {
    fn first(&self, nonterm: Nonterm) -> Vec<Term> {
        self.first_map
            .get(&nonterm)
            .expect("Nonterm non-existent")
            .clone()
    }

    fn follow(&self, nonterm: Nonterm) -> Vec<Term> {
        self.follow_map
            .get(&nonterm)
            .expect("Nonterm non-existent")
            .clone()
    }

    fn nullable(&self, nonterm: Nonterm) -> bool {
        self.nullable_list.contains(&nonterm)
    }

    fn prods_of(&self, nonterm: Nonterm) -> Vec<Production> {
        self.productions
            .iter()
            .filter(|prod| prod.s == nonterm)
            .cloned()
            .collect()
    }

    fn symbols(&self) -> Vec<Symbol> {
        self.all_symbols.clone()
    }
}

impl<S: LRState> LRGrammar<S> {
    pub fn build(init_nonterm: Nonterm, mut productions: Vec<Production>) -> Option<LRGrammar<S>> {
        // Initial production, S' -> S$
        productions.insert(0, Production { s: 0, b: vec![Symbol::Nonterm(init_nonterm), Symbol::Term(b'$')] });

        let grammar = Grammar::build(init_nonterm, productions);

        let init_state = <S as LRState>::initial_state(grammar.productions[0].clone(), &grammar);
        let states = <S as LRState>::all_states(init_state.clone(), &grammar);
        let mut action = BTreeMap::new();
        
        for state in &states {
            let curr_action = state.action(&grammar);
            match curr_action {
                Some(moves) => {
                    for (term, curr_move) in moves {
                        action.insert((state.clone(), term), curr_move);
                    }
                },
                None => return None,
            }
        }

        Some(LRGrammar {
            grammar,
            init_state,
            states,
            action,
        })
    }

    pub fn parse(&self, mut input: &[u8]) -> Option<Vec<Vec<Symbol>>> {
        let mut stack = Vec::new();
        stack.push((self.init_state.clone(), Symbol::Term(b' ')));
        let mut result = Vec::new();
        result.push(input.iter().map(|&c| Symbol::Term(c)).collect());
        println!("{}", String::from_utf8(input.to_vec()).unwrap());
        loop {
            let top_state = stack.last().unwrap().0.clone();
            let next_input = if input.is_empty() { b'$' } else { input[0] };
            let curr_move = self.action.get(&(top_state, next_input));
            match curr_move {
                Some(curr_move) => match curr_move {
                    &Move::Shift(ref state_added) => {
                        stack.push((state_added.clone(), Symbol::Term(next_input)));
                        if next_input == b'$' {
                            result.reverse();
                            return Some(result);
                        } else {
                            input = &input[1..];
                        }
                    },
                    &Move::Reduce(ref prod) => {
                        for _ in 0..prod.b.len() {
                            if stack.pop().is_none() {
                                panic!("Number of states on the stack too small");
                            }
                        }
                        let state_added = stack.last()
                            .expect("The stack should not be empty")
                            .0
                            .go_to(Symbol::Nonterm(prod.s), &self.grammar);
                        stack.push((state_added, Symbol::Nonterm(prod.s)));

                        let mut stack_symbols = stack
                            .iter()
                            .skip(1)
                            .map(|&(_, sym)| sym)
                            .collect::<Vec<_>>();
                        stack_symbols.extend(input.iter().map(|&c| Symbol::Term(c)));
                        result.push(stack_symbols);
                    }
                },
                None => { return None },
            }
        }
    }

    pub fn get_init_nonterm(&self) -> Nonterm {
        self.grammar.init_nonterm
    }

    pub fn get_states(&self) -> Vec<S> {
        self.states.clone()
    }
}