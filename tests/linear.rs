use std::{
    collections::BTreeMap,
    fmt::{Display, Error, Formatter},
    str::FromStr,
};

use egg::*;

define_language! {
  enum SimpleMath {
    "+" = Add([Id; 2]),
    "x" = Mul([Id;2]),
    Num(i32),
    Func(Symbol,Vec<Id>),
  }
}

// 3 * f(x) + 2 * f(y) + 1
// coefs: {x: 3, y: 2}, constant: 1
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
struct LinExp {
    coefs: BTreeMap<Id, i32>,
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

    fn mul(&self, other: &LinExp) -> Option<LinExp> {
        if self.coefs.iter().all(|(_k, v)| *v == 0) {
            Some(LinExp {
                coefs: other
                    .coefs
                    .iter()
                    .map(|(k, v)| (*k, *v * self.constant))
                    .collect::<BTreeMap<Id, i32>>(),
                constant: self.constant * other.constant,
            })
        } else if other.coefs.iter().all(|(_k, v)| *v == 0) {
            Some(LinExp {
                coefs: self
                    .coefs
                    .iter()
                    .map(|(k, v)| (*k, *v * other.constant))
                    .collect::<BTreeMap<Id, i32>>(),
                constant: self.constant * other.constant,
            })
        } else {
            None
        }
    }

    fn prune(&mut self) {
        self.coefs.retain(|_k, v| *v != 0)
    }
}

#[derive(Default)]
struct LinearArith;
impl Analysis<SimpleMath> for LinearArith {
    type Data = Option<LinExp>;

    fn merge(&mut self, to: &mut Self::Data, from: Self::Data) -> DidMerge {
        egg::merge_max(to, from)
    }

    fn make(egraph: &mut EGraph<SimpleMath, Self>, enode: &SimpleMath, id: Id) -> Self::Data {
        let x = |i: &Id| egraph[*i].data.clone();

        match enode {
            SimpleMath::Num(n) => Some(LinExp {
                coefs: BTreeMap::new(),
                constant: *n,
            }),
            SimpleMath::Mul([a, b]) => x(a)?.mul(&x(b)?),
            SimpleMath::Add([a, b]) => Some(x(a)?.add(&x(b)?)),
            SimpleMath::Func(_f, _args) => Some(LinExp {
                coefs: std::iter::once((id, 1)).collect::<BTreeMap<_, _>>(),
                constant: 0,
            }),
        }
    }

    fn modify(egraph: &mut EGraph<SimpleMath, Self>, id: Id) {
        // let added = egraph.add(egraph[id].data.clone());
        if let Some(linexp) = &egraph[id].data {
            let added = egraph.add_expr(&linexp.to_rec_expr());
            egraph.union(id, added);
        }
    }
}
