use ast::VariableDefinition;
use parser::Spanning;
use validation::{ValidatorContext, Visitor};

pub struct UniqueVariableNames {}

pub fn factory() -> UniqueVariableNames {
    UniqueVariableNames {}
}

impl<'a> Visitor<'a> for UniqueVariableNames {
    fn enter_variable_definition(
        &mut self,
        ctx: &mut ValidatorContext<'a>,
        &(ref var_name, ref var_def): &'a (Spanning<&'a str>, VariableDefinition),
    ) {
        if let Some(var_type) = ctx.schema
            .concrete_type_by_name(var_def.var_type.item.innermost_name())
        {
            if !var_type.is_input() {
                ctx.report_error(
                    &error_message(var_name.item, &format!("{}", var_def.var_type.item)),
                    &[var_def.var_type.start.clone()],
                );
            }
        }
    }
}

fn error_message(var_name: &str, type_name: &str) -> String {
    format!(
        "Variable \"{}\" cannot be of non-input type \"{}\"",
        var_name, type_name
    )
}

#[cfg(test)]
mod tests {
    use super::{error_message, factory};

    use parser::SourcePosition;
    use validation::{expect_fails_rule, expect_passes_rule, RuleError};

    #[test]
    fn input_types_are_valid() {
        expect_passes_rule(
            factory,
            r#"
          query Foo($a: String, $b: [Boolean!]!, $c: ComplexInput) {
            field(a: $a, b: $b, c: $c)
          }
        "#,
        );
    }

    #[test]
    fn output_types_are_invalid() {
        expect_fails_rule(
            factory,
            r#"
          query Foo($a: Dog, $b: [[CatOrDog!]]!, $c: Pet) {
            field(a: $a, b: $b, c: $c)
          }
        "#,
            &[
                RuleError::new(
                    &error_message("a", "Dog"),
                    &[SourcePosition::new(25, 1, 24)],
                ),
                RuleError::new(
                    &error_message("b", "[[CatOrDog!]]!"),
                    &[SourcePosition::new(34, 1, 33)],
                ),
                RuleError::new(
                    &error_message("c", "Pet"),
                    &[SourcePosition::new(54, 1, 53)],
                ),
            ],
        );
    }
}
