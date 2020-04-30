pub mod lr_grammar;
pub mod grammar;

use std::collections::HashMap;
use std::mem::MaybeUninit;
use std::io::Read;

use lr_grammar::LRGrammar;
use lr_grammar::states::canonical_lr_state::CanonicalLRState;
use grammar::types::{Nonterm, Term, Production, Symbol};

fn read_grammar(input: &str) -> (Vec<Production>, Vec<String>) {
    let mut nonterms = HashMap::new();
    let mut get_index = |s: String| -> usize {
        if !nonterms.contains_key(&s) {
            let index = nonterms.len();
            nonterms.insert(s, index);
            index
        } else {
            *nonterms.get(&s).unwrap()
        }
    };
    let mut productions = Vec::new();
    for line in input.split('\n') {
        if line.trim().is_empty() {
            continue;
        }
        let parts: Vec<&str> = line
            .trim()
            .split(' ')
            .filter(|p| p.trim().len() > 0)
            .collect();
        if parts.is_empty() {
            continue;
        }
        if parts.len() < 3 || parts[1] != "->" {
            panic!("Bad production");
        }
        if !parts[0].chars().next().unwrap().is_uppercase() {
            panic!("Production with nonterm that starts with an non-uppercase character");
        }
        let s = get_index(parts[0].to_string());
        let mut b = Vec::new();
        for c in &parts[2..] {
            let bytes: &[u8] = c.as_bytes();
            if bytes.len() == 1 && !(bytes[0] as char).is_uppercase() {
                b.push(Symbol::Term(bytes[0]));
            } else {
                b.push(Symbol::Nonterm(get_index(c.to_string())));
            }
        }
        productions.push(Production { s, b });
    }
    let mut ret_vec = Vec::with_capacity(nonterms.len());
    for _ in 0..nonterms.len() {
        ret_vec.push(MaybeUninit::<String>::uninit());
    }
    for (k, v) in nonterms.into_iter() {
        unsafe { ret_vec[v].as_mut_ptr().write(k) };
    }
    let ret_vec = ret_vec
        .into_iter()
        .map(|x| unsafe { x.assume_init() })
        .collect();
    (productions, ret_vec)
}

fn main() {
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input).expect("Error reading");
    let (productions, nonterms) = read_grammar(&input);
    let init_nonterm = productions[0].s;
    let lr_grammar: LRGrammar<CanonicalLRState> = LRGrammar::build(init_nonterm, productions)
        .expect("Not an LR(1) grammar");
    let word = b"n+n*n+n*n";
    match lr_grammar.parse(word) {
        None => println!("Doesn't match!"),
        Some(rightmost_derivation) => {
            println!("Matches!");
            for step in rightmost_derivation {
                for s in step {
                    match s {
                        Symbol::Nonterm(t) => print!("{} ", nonterms[t]),
                        Symbol::Term(t) => print!("{} ", t as char),
                    }
                }
                println!();
            }
        }
    }
}
