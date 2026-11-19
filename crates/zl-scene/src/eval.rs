use crate::ObjectId;
use crate::Value;
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Default)]
pub struct EvalContext {
    pub objects: BTreeMap<ObjectId, Value>,
    pub variables: Vec<(String, Value)>,
    pub time: f64,
}

impl EvalContext {
    pub fn variable(&self, name: &str) -> Option<&Value> {
        for (var_name, value) in self.variables.iter().rev() {
            if var_name == name {
                return Some(value);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx_with(vars: &[(&str, f64)]) -> EvalContext {
        EvalContext {
            variables: vars
                .iter()
                .map(|(name, v)| (name.to_string(), Value::Real(*v)))
                .collect(),
            ..EvalContext::default()
        }
    }

    #[test]
    fn empty_context_has_no_variable() {
        assert_eq!(EvalContext::default().variable("x"), None);
    }

    #[test]
    fn finds_a_single_variable() {
        let ctx = ctx_with(&[("x", 0.3)]);
        assert_eq!(ctx.variable("x"), Some(&Value::Real(0.3)));
    }

    #[test]
    fn unknown_name_returns_none() {
        let ctx = ctx_with(&[("x", 0.3), ("n", 2.0)]);
        assert_eq!(ctx.variable("t"), None);
    }

    #[test]
    fn inner_variable_shadows_outer_one() {
        let ctx = ctx_with(&[("n", 1.0), ("n", 2.0)]);
        assert_eq!(ctx.variable("n"), Some(&Value::Real(2.0)));
    }

    #[test]
    fn outer_variable_reappears_after_pop() {
        let mut ctx = ctx_with(&[("n", 1.0), ("n", 2.0)]);
        ctx.variables.pop();
        assert_eq!(ctx.variable("n"), Some(&Value::Real(1.0)));
    }

    #[test]
    fn other_variables_stay_visible_under_shadowing() {
        let ctx = ctx_with(&[("x", 0.5), ("n", 1.0), ("n", 2.0)]);
        assert_eq!(ctx.variable("x"), Some(&Value::Real(0.5)));
    }

    #[test]
    fn names_must_match_exactly() {
        let ctx = ctx_with(&[("n", 1.0)]);
        assert_eq!(ctx.variable("nn"), None);
        assert_eq!(ctx.variable(""), None);
        assert_eq!(ctx.variable("N"), None);
    }
}
