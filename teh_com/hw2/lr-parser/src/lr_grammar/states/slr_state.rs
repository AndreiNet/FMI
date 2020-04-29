use std::cmp::{PartialOrd, Ord, PartialEq, Eq};
use std::collections::{BTreeSet, BTreeMap, VecDeque};

use crate::lr_grammar::types::{LRState, GrammarInfo, Move};
use crate::grammar::types::{Production, Symbol, Term};

#[derive(Clone, PartialOrd, PartialEq, Ord, Eq)]
struct SuperProduction {
    prod: Production,
    prod_pos: usize,
}

#[derive(Clone, PartialOrd, PartialEq, Ord, Eq)]
struct SLRState {
    sprods: Vec<SuperProduction>,
}

impl LRState for SLRState {
    fn closure<G: GrammarInfo>(&self, info: &G) -> SLRState {
        let mut result_sprods = BTreeSet::new();
        let mut q = VecDeque::new();
        for sprod in &self.sprods {
            result_sprods.insert(sprod.clone());
            q.push_back(sprod.clone());
        }
        while !q.is_empty() {
            let first = q.pop_front().unwrap();
            if first.prod_pos < first.prod.b.len() {
                let extend_sym = first.prod.b[first.prod_pos];
                if let Symbol::Nonterm(t) = extend_sym {
                    for new_prod in info.prods_of(t) {
                        let new_sprod = SuperProduction {
                            prod: new_prod.clone(),
                            prod_pos: 0,
                        };
                        if result_sprods.insert(new_sprod.clone()) {
                            q.push_back(new_sprod);
                        }
                    }
                }
            }
        }
        SLRState {
            sprods: result_sprods.into_iter().collect(),
        }
    }

    fn initial_state<G: GrammarInfo>(initial_prod: Production, info: &G) -> SLRState {
        let sprod = SuperProduction {
            prod: initial_prod,
            prod_pos: 0,
        };
        let state = SLRState { sprods: vec![sprod] };
        state.closure(info)
    }

    fn go_to<G: GrammarInfo>(&self, sym: Symbol, info: &G) -> SLRState {
        let mut new_sprods = BTreeSet::new();
        for sprod in &self.sprods {
            if sprod.prod_pos < sprod.prod.b.len() && sprod.prod.b[sprod.prod_pos] == sym {
                let new_sprod = SuperProduction {
                    prod: sprod.prod.clone(),
                    prod_pos: sprod.prod_pos + 1,
                };
                new_sprods.insert(new_sprod);
            }
        }
        SLRState { sprods: new_sprods.into_iter().collect() }.closure(info)
    }

    fn action<G: GrammarInfo>(&self, info: &G) -> Option<Vec<(Term, Move<SLRState>)>> {
        let all_symbols = info.symbols();
        let mut moves: BTreeMap<Term, Move<_>> = BTreeMap::new();
        let all_terms: Vec<Term> = all_symbols
            .iter()
            .filter_map(|&sym| match sym { Symbol::Term(t) => Some(t), _ => None, })
            .collect();
        for sym in all_terms {
            let new_state = self.go_to(Symbol::Term(sym), info);
            if !new_state.sprods.is_empty() {
                moves.insert(sym, Move::Shift(new_state));
            }
        }
        for sprod in &self.sprods {
            if sprod.prod_pos == sprod.prod.b.len() {
                for c in info.follow(sprod.prod.s) {
                    let prev_value = moves.insert(c, Move::Reduce(sprod.prod.clone()));
                    // There is a confilct for the move
                    if prev_value.is_some() {
                        return None;
                    }
                }
            }
        }
        Some(moves.into_iter().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lr_grammar::LRGrammar;

    #[test]
    fn non_slr() {
        // The grammar:
        // S -> Aa
        // S -> bAc
        // S -> dc
        // S -> bda
        // A -> d
        let productions = vec![
            Production { s: 0, b: vec![Symbol::Nonterm(1), Symbol::Term(b'a')] },
            Production { s: 0, b: vec![Symbol::Term(b'b'), Symbol::Nonterm(1), Symbol::Term(b'c')] },
            Production { s: 0, b: vec![Symbol::Term(b'd'), Symbol::Term(b'c')] },
            Production { s: 0, b: vec![Symbol::Term(b'b'), Symbol::Term(b'd'), Symbol::Term(b'a')] },
            Production { s: 1, b: vec![Symbol::Term(b'd')] }
        ];
        let init_nonterm = 0;
        let parser = LRGrammar::<SLRState>::build(init_nonterm, productions);
        assert!(parser.is_none());
    }

    #[test]
    fn parse1() {
        // The grammar:
        // E -> E + T
        // E -> T
        // T -> T * F
        // T -> F
        // F -> n
        let productions = vec![
            Production { s: 0, b: vec![Symbol::Nonterm(0), Symbol::Term(b'+'), Symbol::Nonterm(1)] },
            Production { s: 0, b: vec![Symbol::Nonterm(1)] },
            Production { s: 1, b: vec![Symbol::Nonterm(1), Symbol::Term(b'*'), Symbol::Nonterm(2)] },
            Production { s: 1, b: vec![Symbol::Nonterm(2)] },
            Production { s: 2, b: vec![Symbol::Term(b'n')] },
        ];
        let init_nonterm = 0;
        let parser: LRGrammar<SLRState> = LRGrammar::build(init_nonterm, productions)
            .expect("Should be SLR(1)");
        assert_eq!(parser.parse(b"n*n*n"), true);
        assert_eq!(parser.parse(b"n+n+n"), true);
        assert_eq!(parser.parse(b"n+n*n+n*n"), true);
        assert_eq!(parser.parse(b"n+n*"), false);
        assert_eq!(parser.parse(b"n+*n"), false);
    }
}