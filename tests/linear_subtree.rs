use std::collections::BTreeMap;

use egg::*;

define_language! {
  enum SimpleMath {
    "+" = Add([Id; 2]),
    Num(i32),
    Symbol(Symbol),
  }
}

// 3 * x + 2 * y + 1
// coefs: {x: 3, y: 2}, constant: 1
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
struct LinExp {
    coefs: BTreeMap<Symbol, i32>,
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
}

#[derive(Default)]
struct LinearArith;
impl Analysis<SimpleMath> for LinearArith {
    type Data = Option<LinExp>;

    fn merge(&mut self, to: &mut Self::Data, from: Self::Data) -> DidMerge {
        egg::merge_max(to, from)
    }

    fn make(egraph: &mut EGraph<SimpleMath, Self>, enode: &SimpleMath) -> Self::Data {
        let x = |i: &Id| egraph[*i].data.clone();
        match enode {
            SimpleMath::Num(n) => Some(LinExp {
                coefs: BTreeMap::new(),
                constant: *n,
            }),
            SimpleMath::Add([a, b]) => Some(x(a)?.add(&x(b)?)),
            SimpleMath::Symbol(sym) => Some(LinExp {
                coefs: BTreeMap::from([(sym.clone(), 1)]),
                constant: 0,
            }),
        }
    }

    fn modify(egraph: &mut EGraph<SimpleMath, Self>, id: Id) {
        if let Some(linexp) = egraph[id].data {
            let added = egraph.add(linexp);
            egraph.union(id, added);
        }
    }
}
