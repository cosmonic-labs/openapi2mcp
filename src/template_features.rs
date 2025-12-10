const START_TOKEN: &str = "// START_OF";
const END_TOKEN: &str = "// END_OF";
const AUTH_FEATURE_TOKEN: &str = "Features.Auth";

#[derive(Debug, Clone, Default)]
pub struct Features {
    pub auth: bool,
}

impl Features {
    fn new() -> Self {
        Self::default()
    }

    fn enable_feature(&mut self, string: &str) {
        let string = string
            .replacen(START_TOKEN, "", 1)
            .replacen(END_TOKEN, "", 1)
            .trim()
            .to_string();
        match string.as_str() {
            AUTH_FEATURE_TOKEN => self.auth = true,
            _ => panic!("Unknown feature: {}", string),
        }
    }

    fn disable_feature(&mut self, string: &str) {
        let string = string
            .replacen(START_TOKEN, "", 1)
            .replacen(END_TOKEN, "", 1)
            .trim()
            .to_string();
        match string.as_str() {
            AUTH_FEATURE_TOKEN => self.auth = false,
            _ => panic!("Unknown feature: {}", string),
        }
    }
}

pub fn handle_template_features(needed: &Features, input: &str) -> String {
    // what blocks are we currently in
    let mut active_blocks = Features::new();

    input
        .lines()
        .filter_map(|line| {
            if line.trim_start().starts_with(START_TOKEN) {
                active_blocks.enable_feature(line);
                None
            } else if line.trim_start().starts_with(END_TOKEN) {
                active_blocks.disable_feature(line);
                None
            } else if active_blocks.auth {
                if needed.auth {
                    Some(line.replacen("// ", "", 1))
                } else {
                    None
                }
            } else {
                Some(line.to_string())
            }
        })
        .collect::<Vec<String>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_replacement() {
        const INPUT: &str = r#"
            // random js code
        "#;
        const EXPECT: &str = INPUT;
        let output = super::handle_template_features(&Features { auth: true }, INPUT);
        assert_eq!(output, EXPECT);
    }

    #[test]
    fn auth_enabled() {
        const INPUT: &str = r#"
            // START_OF Features.Auth
            // // set var
            // const foo = "bar";
            // END_OF Features.Auth
            // log 42
            console.log(42);
            // START_OF Features.Auth
            // const baz = "qux";
            // END_OF Features.Auth
        "#;
        const EXPECT: &str = r#"
            // set var
            const foo = "bar";
            // log 42
            console.log(42);
            const baz = "qux";
        "#;
        let output = super::handle_template_features(&Features { auth: true }, INPUT);
        assert_eq!(output, EXPECT);
    }

    #[test]
    fn auth_disabled() {
        const INPUT: &str = r#"
            // START_OF Features.Auth
            // // set var
            // const foo = "bar";
            // END_OF Features.Auth
            // log 42
            console.log(42);
            // START_OF Features.Auth
            // const baz = "qux";
            // END_OF Features.Auth
        "#;
        const EXPECT: &str = r#"
            // log 42
            console.log(42);
        "#;
        let output = super::handle_template_features(&Features { auth: false }, INPUT);
        assert_eq!(output, EXPECT);
    }

    #[test]
    #[should_panic]
    fn invalid_feature() {
        const INPUT: &str = r#"
            // START_OF Features.Invalid
            // const foo = "bar";
            // END_OF Features.Invalid
        "#;
        let _output = super::handle_template_features(&Features { auth: true }, INPUT);
    }
}
