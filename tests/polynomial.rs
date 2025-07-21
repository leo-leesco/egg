use std::collections::{BTreeMap, BTreeSet, VecDeque};

use egg::*;

define_language! {
    /// represents addition and multiplication of arbitrary multi-variate polynomials, with integer
    /// coefficients
  enum SimpleMath {
    "+" = Add([Id; 2]),
    "x" = Mul([Id;2]),
    Num(i32),
    Var(Symbol),
  }
}

// 3 * x * y + 2 * y + 1
// coefs: {x: 3, y: 2}, constant: 1
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
struct MultiPolynomial {
    coefs: BTreeMap<BTreeSet<Symbol>, i32>,
    constant: i32,
}

impl MultiPolynomial {
    /// removes the zero values
    fn prune(&mut self) -> MultiPolynomial {
        let _ = self.coefs.iter().filter(|(_, v)| **v != 0);
        self.clone()
    }

    fn add(&self, other: &MultiPolynomial) -> MultiPolynomial {
        let mut coefs = self.coefs.clone();
        for (idx, coef) in &other.coefs {
            *coefs.entry(idx.clone()).or_insert(0) += coef;
        }
        MultiPolynomial {
            coefs,
            constant: self.constant + other.constant,
        }
        .prune()
    }

    fn mul(&self, other: &MultiPolynomial) -> MultiPolynomial {
        let mut coefs: BTreeMap<BTreeSet<Symbol>, i32> = BTreeMap::new();
        for (i1, a1) in &self.coefs {
            for (i2, a2) in &other.coefs {
                coefs
                    .entry(i1.union(i2).cloned().collect())
                    .and_modify(|c| *c += *a1 * *a2)
                    .or_insert(*a1 * *a2);
            }
        }

        if self.constant != 0 {
            for (i2, a2) in &other.coefs {
                coefs
                    .entry(i2.clone())
                    .and_modify(|c| *c += self.constant * *a2)
                    .or_insert(self.constant * *a2);
            }
        }
        if other.constant != 0 {
            for (i1, a1) in &self.coefs {
                coefs
                    .entry(i1.clone())
                    .and_modify(|c| *c += other.constant * *a1)
                    .or_insert(other.constant * *a1);
            }
        }

        MultiPolynomial {
            coefs,
            constant: self.constant * other.constant,
        }
        .prune()
    }
}

#[derive(Default)]
struct LinearArith;
impl Analysis<SimpleMath> for LinearArith {
    type Data = MultiPolynomial;

    fn merge(&mut self, to: &mut Self::Data, from: Self::Data) -> DidMerge {
        egg::merge_max(to, from)
    }

    fn make(egraph: &mut EGraph<SimpleMath, Self>, enode: &SimpleMath) -> Self::Data {
        let x = |i: &Id| egraph[*i].data.clone();
        match enode {
            SimpleMath::Num(n) => MultiPolynomial {
                coefs: BTreeMap::new(),
                constant: *n,
            },
            SimpleMath::Var(v) => MultiPolynomial {
                coefs: std::iter::once((std::iter::once(*v).collect(), 1)).collect(),
                constant: 0,
            },
            SimpleMath::Add([a, b]) => x(a).add(&x(b)),
            SimpleMath::Mul([a, b]) => x(a).mul(&x(b)),
        }
    }
}

// the rewriting should now be instantaneous compared to the following rules :
// add-com : (+ ?a ?b) => (+ ?b ?a)
// add-assoc : (+ (+ ?a ?b) ?c) => (+ ?a (+ ?b ?c))
// times-com : (x ?a ?b) => (x ?b ?a)
// times-assoc : (x (x ?a ?b) ?c) => (x ?a (x ?b ?c))
//
// here are some example that should be interesting :
// (x + y) + (x + y) + (x + y) + (x + y) => 4x + 4y
// (x * y) * (x * y) * (x * y) * (x * y) => x^4 * y^4
// 4(x + y) => 4x + 4y
// (x + y)^4 => x^4 + 4x^3y + 6x^2y^2 + 4xy^3 + y^4
// (x + y)^n => sum_{i=0}^n binom ni x^iy^{n-i} with (say) n = 25

fn binom_line(n: usize) -> VecDeque<usize> {
    if n == 1 {
        VecDeque::from([1])
    } else {
        let mut prev_line = binom_line(n - 1);
        let mut offset = prev_line.clone();
        offset.push_front(0);
        prev_line.push_back(0);
        offset.iter().zip(prev_line).map(|(a, b)| a + b).collect()
    }
}
