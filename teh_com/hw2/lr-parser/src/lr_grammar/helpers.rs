use std::collections::BTreeSet;

use super::types::*;
use super::super::grammar::types::*;

/// Returns first terms and if it can be empty
pub fn get_first<G: GrammarInfo>(symbols: &[Symbol], info: &G) ->
    (Vec<Term>, bool) {
    let mut first = BTreeSet::new();
    let mut nullable = true;
    for symbol in symbols {
        match symbol {
            &Symbol::Nonterm(t) => {
                first.extend(info.first(t));
                if !info.nullable(t) {
                    nullable = false;
                    break;
                }
            }
            &Symbol::Term(t) => {
                first.insert(t);
                nullable = false;
                break;
            }
        }
    }
    (first.into_iter().collect(), nullable)
}
