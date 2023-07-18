// pub mod editor_dock_tab;
pub mod request_ui;

#[cfg(test)]
mod tests {
    use super::*;
    use minijinja::{context, Environment};

    #[test]
    fn test_expr() {
        let env = Environment::new();
        let expr = env.compile_expression("number + 42").unwrap();
        let result = expr.eval(context!(number => 23)).unwrap();
        println!("{result}");
        let expr = env.compile_expression("number < 42").unwrap();
        let result = expr.eval(context!(number => result)).unwrap();
        println!("{result}");
    }
}
