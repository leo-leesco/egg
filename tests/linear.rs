use std::{
    collections::BTreeMap,
    fmt::{Display, Error, Formatter},
    str::FromStr,
};

use egg::*;
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct LinearSymbol(Symbol, i32);

define_language! {
  enum SimpleMath {
    "+" = Add([Id; 2]),
    Num(i32),
    Func(LinearSymbol, Vec<Id>),
  }
}

impl FromStr for LinearSymbol {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.splitn(2, ' ');
        let sym = parts.next().ok_or(())?.to_string();
        let arg = parts.next().and_then(|a| a.parse().ok()).ok_or(())?;
        Ok(LinearSymbol(Symbol::from(sym), arg))
    }
}

impl Display for LinearSymbol {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{} {}", self.0, self.1)
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
    fn add(&self, egraph: &EGraph<SimpleMath, LinearArith>, other: &LinExp) -> LinExp {
        let mut coefs = self.coefs.clone();
        for (sym, coef) in &other.coefs {
            *coefs.entry(egraph.find(*sym)).or_insert(0) += coef;
        }
        LinExp {
            coefs,
            constant: self.constant + other.constant,
        }
    }

    fn to_rec_expr(&self, egraph: &EGraph<SimpleMath, LinearArith>) -> RecExpr<SimpleMath> {}
}

#[derive(Default)]
struct LinearArith;
impl Analysis<SimpleMath> for LinearArith {
    type Data = Option<LinExp>;

    fn merge(&mut self, to: &mut Self::Data, from: Self::Data) -> DidMerge {
        egg::merge_max(to, from)
    }

    fn make(egraph: &mut EGraph<SimpleMath, Self>, enode: &SimpleMath, id: Id) -> Self::Data {
        let x = |e: &EGraph<SimpleMath, Self>, i: &Id| e[*i].data.clone();
        match enode {
            SimpleMath::Num(n) => Some(LinExp {
                coefs: BTreeMap::new(),
                constant: *n,
            }),
            SimpleMath::Add([a, b]) => Some(x(egraph, a)?.add(egraph, &x(egraph, b)?)),
            SimpleMath::Func(LinearSymbol(_sym, coef), _args) => {
                if *coef == 1 {
                    None
                } else {
                    egraph.add_expr(todo!());
                    Some(LinExp {
                        coefs: std::iter::once((id, *coef)).collect::<BTreeMap<_, _>>(),
                        constant: 0,
                    })
                }
            }
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
