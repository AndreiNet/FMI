use std::cmp::{PartialOrd, PartialEq, Ord, Eq};
use std::collections::{BTreeMap, BTreeSet};

use super::super::grammar::types::*;

pub type FirstMap<'a> = &'a BTreeMap<Nonterm, Vec<Term>>;
pub type NullableNonterms<'a> = &'a BTreeSet<Nonterm>;

pub trait GrammarInfo {
    fn first(&self, nonterm: Nonterm) -> Vec<Term>;
    fn follow(&self, nonterm: Nonterm) -> Vec<Term>;
    fn nullable(&self, nonterm: Nonterm) -> bool;
    fn prods_of(&self, nonterm: Nonterm) -> Vec<Production>;
    fn symbols(&self) -> Vec<Symbol>;
}

pub trait LRState: Ord + Eq + Clone + Sized  {
    fn closure<G: GrammarInfo>(&self, info: &G) -> Self;
    fn all_states<G: GrammarInfo>(from: Self, info: &G) -> Vec<Self>;
    fn initial_state<G: GrammarInfo>(initial_prod: Production, info: &G) -> Self;
    fn go_to<G: GrammarInfo>(&self, sym: Symbol, info: &G) -> Self;
    fn action<G: GrammarInfo>(&self, info: &G) -> Option<Vec<(Term, Move<Self>)>>;
}

#[derive(Clone, PartialOrd, PartialEq, Ord, Eq)]
pub enum Move<S: LRState> {
    Shift(S),
    Reduce(Production),
}
