use std::collections::HashMap;
use std::iter::Map;

use egg::*;

define_language! {
  enum SimpleMath {
    "+" = Add([Id; 2]),
    "*" = Mul([Id; 2]),
    Num(i32),
    Symbol(Symbol),
  }
}

// 3 * f(x) + 2 * f(y) + 1
// coefs: {x: 3, y: 2}, constant: 1
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct LinExp {
    coefs: HashMap<Symbol, i32>,
    constant: i32,
}

impl LinExp {
    fn add(&self, other: &LinExp) -> LinExp {
        let mut coefs = self.coefs.clone();
        for (sym, coef) in &other.coefs {
            *coefs.entry(*sym).or_insert(0) += coef;
        }
        LinExp {
            coefs,
            constant: self.constant + other.constant,
        }
    }

    // fn mul(&self, other: &LinExp) -> Option<LinExp> {
    //     // let mut coefs = Map::new();
    //     // for (sym1, coef1) in &self.coefs {
    //     //     for (sym2, coef2) in &other.coefs {
    //     //         *coefs.entry(*sym1 + *sym2).or_insert(0) += coef1 * coef2;
    //     //     }
    //     // }
    //     // LinExp {
    //     //     coefs,
    //     //     constant: self.constant * other.constant,
    //     // }
    // }
}

#[derive(Default)]
struct LinearArith;
impl Analysis<SimpleMath> for LinearArith {
    type Data = Option<LinExp>;

    fn merge(&mut self, to: &mut Self::Data, from: Self::Data) -> DidMerge {
        egg::merge_max(to, from)
    }

    fn make(egraph: &mut EGraph<SimpleMath, Self>, enode: &SimpleMath) -> Self::Data {
        let x = |i: &Id| egraph[*i].data;
        match enode {
            SimpleMath::Num(n) => Some(LinExp {
                coefs: HashMap::new(),
                constant: n,
            }),
            SimpleMath::Add([a, b]) => Some(x(a)?.add(x(b)?)),
            SimpleMath::Mul([a, b]) => Some(x(a)?.add(x(b)?)),
            SimpleMath::Symbol(sym) => Some(LinExp {
                coefs: Map::from([(sym.clone(), 1)]),
                constant: 0,
            }),
        }
    }

    fn modify(egraph: &mut EGraph<SimpleMath, Self>, id: Id) {
        if let Some(linexp) = egraph[id].data {
            let added = egraph.add(..);
            egraph.union(id, added);
        }
    }
}
