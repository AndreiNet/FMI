use std::cmp::{PartialOrd, Ord, PartialEq, Eq};

use crate::grammar::types::*;
use crate::lr_grammar::{types::*, helpers::*};

use std::collections::{BTreeSet, BTreeMap, VecDeque};

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord)]
struct SuperProduction {
    prod: Production,
    prod_pos: usize,
    c: Term,
}

#[derive(Clone, PartialOrd, PartialEq, Ord, Eq)]
pub struct CanonicalLRState {
    sprods: Vec<SuperProduction>,
}

impl LRState for CanonicalLRState {
    fn closure<G: GrammarInfo>(&self, info: &G) -> CanonicalLRState {
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
                    let (mut new_cs, nullable) = get_first(&first.prod.b[(first.prod_pos + 1)..], info);
                    if nullable && !new_cs.contains(&first.c) {
                        new_cs.push(first.c);
                    }
                    for new_prod in info.prods_of(t) {
                        for &new_c in &new_cs {
                            let new_sprod = SuperProduction {
                                prod: new_prod.clone(),
                                prod_pos: 0,
                                c: new_c,
                            };
                            if result_sprods.insert(new_sprod.clone()) {
                                q.push_back(new_sprod);
                            }
                        }
                    }
                }
            }
        }
        CanonicalLRState {
            sprods: result_sprods.into_iter().collect(),
        }
    }

    fn go_to<G: GrammarInfo>(&self, sym: Symbol, info: &G) -> Self {
        let mut new_sprods = BTreeSet::new();
        for sprod in &self.sprods {
            if sprod.prod_pos < sprod.prod.b.len() && sprod.prod.b[sprod.prod_pos] == sym {
                let new_sprod = SuperProduction {
                    prod: sprod.prod.clone(),
                    prod_pos: sprod.prod_pos + 1,
                    c: sprod.c,
                };
                new_sprods.insert(new_sprod);
            }
        }
        CanonicalLRState { sprods: new_sprods.into_iter().collect() }.closure(info)
    }

    fn initial_state<G: GrammarInfo>(initial_prod: Production, info: &G) -> Self {
        let sprod = SuperProduction {
            prod: initial_prod,
            prod_pos: 0,
            c: b'$',
        };
        let state = CanonicalLRState { sprods: vec![sprod] };
        state.closure(info)
    }

    fn action<G: GrammarInfo>(&self, info: &G) -> Option<Vec<(Term, Move<CanonicalLRState>)>> {
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
                let prev_value = moves.insert(sprod.c, Move::Reduce(sprod.prod.clone()));
                // There is a confilct for the move
                if prev_value.is_some() {
                    return None;
                }
            }
        }
        Some(moves.into_iter().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn closure_1() {
        // The grammar:
        // S -> AB
        // A -> a
        // B -> Cb
        // C -> c | #
        let productions = vec![
            Production { s: 0, b: vec![Symbol::Nonterm(1), Symbol::Nonterm(2)] },
            Production { s: 1, b: vec![Symbol::Term(b'a')] },
            Production { s: 2, b: vec![Symbol::Nonterm(3), Symbol::Term(b'b')] },
            Production { s: 3, b: vec![Symbol::Term(b'c')] },
            Production { s: 3, b: vec![]},
        ];
        let init_nonterm = 0;
        let grammar = Grammar::build(init_nonterm, productions.clone());
        let state = CanonicalLRState {
            sprods: vec![
                SuperProduction { prod: productions[0].clone(), prod_pos: 1, c: b'v' },
            ],
        };
        let closure = state.closure(&grammar);
        assert_eq!(&closure.sprods, &vec![
            SuperProduction { prod: productions[0].clone(), prod_pos: 1, c: b'v' },
            SuperProduction { prod: productions[2].clone(), prod_pos: 0, c: b'v' },
            SuperProduction { prod: productions[4].clone(), prod_pos: 0, c: b'b' },
            SuperProduction { prod: productions[3].clone(), prod_pos: 0, c: b'b' },
        ]);

        let state = CanonicalLRState {
            sprods: vec![
                SuperProduction { prod: productions[0].clone(), prod_pos: 0, c: b'v'},
            ],
        };
        let closure = state.closure(&grammar);
        assert_eq!(&closure.sprods, &vec![
            SuperProduction { prod: productions[0].clone(), prod_pos: 0, c: b'v' },
            SuperProduction { prod: productions[1].clone(), prod_pos: 0, c: b'b' },
            SuperProduction { prod: productions[1].clone(), prod_pos: 0, c: b'c' },
        ]);
    }

    #[test]
    fn action_1() {
        // The grammar:
        // S -> AB
        // A -> a
        // B -> Cb
        // C -> c | #
        let productions = vec![
            Production { s: 0, b: vec![Symbol::Nonterm(1), Symbol::Nonterm(2)] },
            Production { s: 1, b: vec![Symbol::Term(b'a')] },
            Production { s: 2, b: vec![Symbol::Nonterm(3), Symbol::Term(b'b')] },
            Production { s: 3, b: vec![Symbol::Term(b'c')] },
            Production { s: 3, b: vec![]},
        ];
        let init_nonterm = 0;
        let grammar = Grammar::build(init_nonterm, productions.clone());
    }

    use crate::lr_grammar::LRGrammar;
    #[test]
    fn parse_1() {
        // The grammar:
        // S -> AB
        // A -> a
        // B -> Cb
        // C -> c | #
        let productions = vec![
            Production { s: 0, b: vec![Symbol::Nonterm(1), Symbol::Nonterm(2)] },
            Production { s: 1, b: vec![Symbol::Term(b'a')] },
            Production { s: 2, b: vec![Symbol::Nonterm(3), Symbol::Term(b'b')] },
            Production { s: 3, b: vec![Symbol::Term(b'c')] },
            Production { s: 3, b: vec![]},
        ];
        let init_nonterm = 0;
        
        let parser: LRGrammar<CanonicalLRState> = LRGrammar::build(init_nonterm, productions)
            .expect("Should be an LR(1) grammar");
        assert_eq!(parser.parse(b"acb"), true);
        assert_eq!(parser.parse(b"ab"), true);
        assert_eq!(parser.parse(b"a"), false);
        assert_eq!(parser.parse(b"b"), false);
        assert_eq!(parser.parse(b"c"), false);
        assert_eq!(parser.parse(b"ac"), false);
    }

    #[test]
    fn parse_2() {
        // The grammar:
        // S -> AB
        // A -> a
        // B -> CB | D
        // C -> c
        // D -> #
        let productions = vec![
            Production { s: 0, b: vec![Symbol::Nonterm(1), Symbol::Nonterm(2)] },
            Production { s: 1, b: vec![Symbol::Term(b'a')] },
            Production { s: 2, b: vec![Symbol::Nonterm(3), Symbol::Nonterm(2)] },
            Production { s: 2, b: vec![Symbol::Nonterm(4)] },
            Production { s: 3, b: vec![Symbol::Term(b'c')] },
            Production { s: 4, b: vec![]},
        ];
        let init_nonterm = 0;
        let parser: LRGrammar<CanonicalLRState> = LRGrammar::build(init_nonterm, productions)
            .expect("Should be an LR(1) grammar");
        assert_eq!(parser.parse(b"acccc"), true);
        assert_eq!(parser.parse(b"a"), true);
        assert_eq!(parser.parse(b"c"), false);
    }

    #[test]
    fn parse_3() {
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
        let parser: LRGrammar<CanonicalLRState> = LRGrammar::build(init_nonterm, productions)
            .expect("Should be an LR(1) grammar");
        assert_eq!(parser.parse(b"da"), true);
        assert_eq!(parser.parse(b"aa"), false);
        assert_eq!(parser.parse(b"bdc"), true);
    }

    #[test]
    fn parse4() {
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
        let parser: LRGrammar<CanonicalLRState> = LRGrammar::build(init_nonterm, productions)
            .expect("Should be LR(1)");
        assert_eq!(parser.parse(b"n*n*n"), true);
        assert_eq!(parser.parse(b"n+n+n"), true);
        assert_eq!(parser.parse(b"n+n*n+n*n"), true);
        assert_eq!(parser.parse(b"n+n*"), false);
        assert_eq!(parser.parse(b"n+*n"), false);
    }
}