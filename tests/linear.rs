use std::collections::BTreeMap;

use egg::{Rewrite, *};

define_language! {
  enum SimpleMath {
    "+" = Add([Id; 2]),
    "*" = Mul([Id;2]),
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
            coefs: coefs.into_iter().filter(|(_k, v)| *v != 0).collect(),
            constant: self.constant + other.constant,
        }
    }

    fn mul(&self, other: &LinExp) -> Option<LinExp> {
        if self.coefs.iter().all(|(_k, v)| *v == 0) {
            Some(LinExp {
                coefs: other
                    .coefs
                    .iter()
                    .filter_map(|(k, v)| {
                        let mul = *v * self.constant;
                        if mul != 0 {
                            Some((*k, mul))
                        } else {
                            None
                        }
                    })
                    .collect(),
                constant: self.constant * other.constant,
            })
        } else if other.coefs.iter().all(|(_k, v)| *v == 0) {
            Some(LinExp {
                coefs: self
                    .coefs
                    .iter()
                    .filter_map(|(k, v)| {
                        let mul = *v * other.constant;
                        if mul != 0 {
                            Some((*k, mul))
                        } else {
                            None
                        }
                    })
                    .collect(),
                constant: self.constant * other.constant,
            })
        } else {
            None
        }
    }

    fn _prune(&mut self) {
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
                coefs: std::iter::once((id, 1)).collect(),
                constant: 0,
            }),
        }
    }

    fn modify(egraph: &mut EGraph<SimpleMath, Self>, eclass: Id) {
        if let Some(linexp) = &egraph[eclass].data.clone() {
            let mut expr = linexp.coefs.iter().fold(None, |acc, (id, coef)| {
                if *coef == 0 {
                    return acc;
                }

                let coef_node = egraph.add(SimpleMath::Num(*coef));
                let mul = egraph.add(SimpleMath::Mul([coef_node, *id]));

                Some(if let Some(prev) = acc {
                    egraph.add(SimpleMath::Add([prev, mul]))
                } else {
                    mul
                })
            });

            // Step 2: add the constant if non-zero
            if linexp.constant != 0 {
                let const_node = egraph.add(SimpleMath::Num(linexp.constant));
                expr = Some(if let Some(prev) = expr {
                    egraph.add(SimpleMath::Add([prev, const_node]))
                } else {
                    const_node
                });
            }

            // Step 3: union the final expression with the eclass
            if let Some(expr) = expr {
                egraph.union(eclass, expr);
            }
        }
    }
}

#[rustfmt::skip]
fn rules() -> Vec<Rewrite<SimpleMath, ()>> {
    vec![
        rewrite!("comm-add";  "(+ ?a ?b)"        => "(+ ?b ?a)"),
        rewrite!("comm-mul";  "(* ?a ?b)"        => "(* ?b ?a)"),
        rewrite!("assoc-add"; "(+ ?a (+ ?b ?c))" => "(+ (+ ?a ?b) ?c)"),
        rewrite!("assoc-mul"; "(* ?a (* ?b ?c))" => "(* (* ?a ?b) ?c)"),

        rewrite!("sub-canon"; "(- ?a ?b)" => "(+ ?a (* -1 ?b))"),

        rewrite!("zero-add"; "(+ ?a 0)" => "?a"),
        rewrite!("zero-mul"; "(* ?a 0)" => "0"),
        rewrite!("one-mul";  "(* ?a 1)" => "?a"),

        rewrite!("add-zero"; "?a" => "(+ ?a 0)"),
        rewrite!("mul-one";  "?a" => "(* ?a 1)"),

        rewrite!("cancel-sub"; "(- ?a ?a)" => "0"),
        rewrite!("distribute"; "(* ?a (+ ?b ?c))"        => "(+ (* ?a ?b) (* ?a ?c))"),
        rewrite!("factor"    ; "(+ (* ?a ?b) (* ?a ?c))" => "(* ?a (+ ?b ?c))"),
    ]
}

test_fn! {
    math_associate_adds, [
        rewrite!("comm-add"; "(+ ?a ?b)" => "(+ ?b ?a)"),
        rewrite!("assoc-add"; "(+ ?a (+ ?b ?c))" => "(+ (+ ?a ?b) ?c)"),
    ],
    runner = Runner::default()
        .with_iter_limit(7)
        .with_scheduler(SimpleScheduler),
    "(+ 1 (+ 2 (+ 3 (+ 4 (+ 5 (+ 6 7))))))"
    =>
    "(+ 7 (+ 6 (+ 5 (+ 4 (+ 3 (+ 2 1))))))"
    @check |r: Runner<SimpleMath, ()>| assert_eq!(r.egraph.number_of_classes(), 127)
}
test_fn! {
    math_associate_adds_emt, [ ],
    runner = Runner::<SimpleMath, LinearArith>::default()
        .with_iter_limit(7)
        .with_scheduler(SimpleScheduler),
    "(+ 1 (+ 2 (+ 3 (+ 4 (+ 5 (+ 6 7))))))"
    =>
    "28"
}
test_fn! {
    math_associate_adds_emt2, [ ],
    runner = Runner::<SimpleMath, LinearArith>::default()
        .with_iter_limit(7)
        .with_expr(&"(+ 7 (+ 6 (+ 5 (+ 4 (+ 3 (+ 2 1))))))".parse().unwrap())
        .with_scheduler(SimpleScheduler),
    "(+ 1 (+ 2 (+ 3 (+ 4 (+ 5 (+ 6 7))))))"
    =>
    "(+ 7 (+ 6 (+ 5 (+ 4 (+ 3 (+ 2 1))))))"
}

test_fn! {math_simplify_add, rules(), "(+ x (+ x (+ x x)))" => "(* 4 x)" }
test_fn! {math_simplify_add_emt, Vec::<Rewrite<SimpleMath, LinearArith>>::new(), "(+ x (+ x (+ x x)))" => "(* 4 x)" }

// test_fn! {
//     math_simplify_const, rules(),
//     runner = Runner::<SimpleMath, LinearArith>::default()
//         .with_iter_limit(2)
//         .with_scheduler(SimpleScheduler),
//     "(+ 1 (- a (* (- 2 1) a)))" => "1"
// }
test_fn! {
    math_simplify_const_emt, Vec::<Rewrite<SimpleMath, LinearArith>>::new(),
    "(+ 1 (+ a (* (+ -2 1) a)))" => "1"
}

test_fn! {
    ac_overflow, rules(),
    runner = Runner::<SimpleMath,()>::default()
        .with_expr(&"(+ 20 (+ 19 (+ 18 (+ 17 (+ 16 (+ 15 (+ 14 (+ 13 (+ 12 (+ 11 (+ 10 (+ 9 (+ 8 (+ 7 (+ 6 (+ 5 (+ 4 (+ 3 (+ 2 (+ 1 0)))))))))))))))))))))".parse().unwrap())
        .with_scheduler(SimpleScheduler),
    "(+ 0 (+ 1 (+ 2 (+ 3 (+ 4 (+ 5 (+ 6 (+ 7 (+ 8 (+ 9 (+ 10 (+ 11 (+ 12 (+ 13 (+ 14 (+ 15 (+ 16 (+ 17 (+ 18 (+ 19  20))))))))))))))))))))"
    =>
    "(+ 20 (+ 19 (+ 18 (+ 17 (+ 16 (+ 15 (+ 14 (+ 13 (+ 12 (+ 11 (+ 10 (+ 9 (+ 8 (+ 7 (+ 6 (+ 5 (+ 4 (+ 3 (+ 2 (+ 1 0)))))))))))))))))))))"
}
