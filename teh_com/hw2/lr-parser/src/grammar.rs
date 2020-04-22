pub mod types;

use std::collections::{BTreeSet, BTreeMap};

use types::*;

impl Grammar {
    pub fn build(init_nonterm: Nonterm, productions: Vec<Production>) -> Grammar {
        let mut all_symbols = BTreeSet::new();
        for prod in &productions {
            all_symbols.insert(Symbol::Nonterm(prod.s));
            all_symbols.extend(&prod.b);
        }
        let all_symbols = all_symbols.into_iter().collect();


        let mut nullable_list = BTreeSet::new();
        let mut first_map = BTreeMap::new();
        let mut follow_map = BTreeMap::new();

        // All existent nonterminals must have associated an empty set initially
        for &sym in &all_symbols {
            if let Symbol::Nonterm(t) = sym {
                first_map.insert(t, BTreeSet::new());
                follow_map.insert(t, BTreeSet::new());
            }
        }

        // Compute Nullable(A)
        loop {
            let mut change = false;
            for prod in &productions {
                let curr_nullable = prod.b.iter().filter(|sym| {
                    match sym {
                        &&Symbol::Term(..) => true,
                        &&Symbol::Nonterm(t) => !nullable_list.contains(&t),
                    }
                }).count() == 0;
                if curr_nullable {
                    change |= nullable_list.insert(prod.s);
                }
            }
            if !change {
                break;
            }
        }

        // Compute First(A)
        // Auxiliary set, see bellow
        let mut empty_set = BTreeSet::new();
        loop {
            let mut change = false;
            for prod in &productions {
                // To solve borrowing issues we extract the set First(prod.s) from the map
                let current_first = {
                    let first = first_map.get_mut(&prod.s).expect("Shoud be initialized properly");
                    std::mem::swap(first, &mut empty_set);
                    &mut empty_set
                };
                for &sym in &prod.b {
                    match sym {
                        Symbol::Term(t) => {
                            change |= current_first.insert(t);
                            break;
                        }
                        Symbol::Nonterm(t) => {
                            let sub_first = first_map.get(&t).expect("Should be initialized properly");
                            for elem in sub_first {
                                change |= current_first.insert(*elem);
                            }
                            if !nullable_list.contains(&t) {
                                break;
                            }
                        }
                    }
                }
                drop(current_first);
                {
                    let first = first_map.get_mut(&prod.s).expect("Shoul be initialized properly");
                    std::mem::swap(first, &mut empty_set);
                }
            }
            if !change {
                break;
            }
        }

        // Compute Follow(A)
        loop {
            let mut change = false;
            for prod in &productions {
                let mut curr_follow: BTreeSet<Term> = BTreeSet::new();
                curr_follow.extend(follow_map.get(&prod.s).expect("..."));
                for &sym in prod.b.iter().rev() {
                    // Process changes for the current symbol in the production
                    if let Symbol::Nonterm(t) = sym {
                        let curr_change = follow_map.get_mut(&t).expect("...");
                        for &next in &curr_follow {
                            change |= curr_change.insert(next);
                        }
                    }
                    // Compute next values for the upcoming symbols
                    match sym {
                        Symbol::Nonterm(t) => {
                            if !nullable_list.contains(&t) {
                                curr_follow.clear();
                            }
                            curr_follow.extend(first_map.get(&t).expect("..."));
                        },
                        Symbol::Term(t) => {
                            curr_follow.clear();
                            curr_follow.insert(t);
                        }
                    }
                }
            }

            if !change {
                break;
            }
        }
        let first_map = first_map
            .into_iter()
            .map(|(k, v)| (k, v.into_iter().collect()))
            .collect();
        let follow_map = follow_map
            .into_iter()
            .map(|(k, v)| (k, v.into_iter().collect()))
            .collect();
        
        Grammar {
            init_nonterm,
            productions,
            first_map,
            follow_map,
            nullable_list,
            all_symbols,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn well_building_1() {
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
        let grammar = Grammar::build(init_nonterm, productions);

        // Test Nullable(A)
        assert_eq!(grammar.nullable_list.iter().cloned().collect::<Vec<_>>(), vec![3]);

        // Test First(A)
        assert_eq!(grammar.first_map.get(&0).unwrap(), &vec![b'a']);
        assert_eq!(grammar.first_map.get(&1).unwrap(), &vec![b'a']);
        assert_eq!(grammar.first_map.get(&2).unwrap(), &vec![b'b', b'c']);
        assert_eq!(grammar.first_map.get(&3).unwrap(), &vec![b'c']);

        // Test Follow(A)
        assert_eq!(grammar.follow_map.get(&0).unwrap(), &vec![]);
        assert_eq!(grammar.follow_map.get(&1).unwrap(), &vec![b'b', b'c']);
        assert_eq!(grammar.follow_map.get(&2).unwrap(), &vec![]);
        assert_eq!(grammar.follow_map.get(&3).unwrap(), &vec![b'b']);
    }
}