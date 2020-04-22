use std::cmp::{PartialOrd, PartialEq, Ord, Eq};
use std::collections::{BTreeMap, BTreeSet};

pub type Term = u8;
pub type Nonterm = usize;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub enum Symbol {
    Term(Term),
    Nonterm(Nonterm),
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord)]
pub struct Production {
    pub s: Nonterm,
    pub b: Vec<Symbol>,
}

pub struct Grammar {
    pub init_nonterm: Nonterm,
    pub productions: Vec<Production>,

    pub first_map: BTreeMap<Nonterm, Vec<Term>>,
    pub follow_map: BTreeMap<Nonterm, Vec<Term>>,
    pub nullable_list: BTreeSet<Nonterm>,

    pub all_symbols: Vec<Symbol>,
}