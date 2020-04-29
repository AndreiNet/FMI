use std::cmp::{PartialOrd, PartialEq, Ord, Eq};
use std::collections::{BTreeSet, VecDeque};

use super::super::grammar::types::*;


pub trait GrammarInfo {
    fn first(&self, nonterm: Nonterm) -> Vec<Term>;
    fn follow(&self, nonterm: Nonterm) -> Vec<Term>;
    fn nullable(&self, nonterm: Nonterm) -> bool;
    fn prods_of(&self, nonterm: Nonterm) -> Vec<Production>;
    fn symbols(&self) -> Vec<Symbol>;
}

pub trait LRState: Ord + Eq + Clone + Sized  {
    fn closure<G: GrammarInfo>(&self, info: &G) -> Self;
    fn initial_state<G: GrammarInfo>(initial_prod: Production, info: &G) -> Self;
    fn go_to<G: GrammarInfo>(&self, sym: Symbol, info: &G) -> Self;

    fn action<G: GrammarInfo>(&self, info: &G) -> Option<Vec<(Term, Move<Self>)>>;

    fn all_states<G: GrammarInfo>(from: Self, info: &G) -> Vec<Self> {
        let symbols = info.symbols();

        let mut states = BTreeSet::new();
        let mut q = VecDeque::new();
        states.insert(from.clone());
        q.push_back(from);
    
        while !q.is_empty() {
            let first = q.pop_front().unwrap();
            for &symbol in &symbols {
                let new_state = first.go_to(symbol, info);
                if states.insert(new_state.clone()) {
                    q.push_back(new_state);
                }
            }
        }
        states.into_iter().collect()
    }
}

#[derive(Clone, PartialOrd, PartialEq, Ord, Eq)]
pub enum Move<S: LRState> {
    Shift(S),
    Reduce(Production),
}
