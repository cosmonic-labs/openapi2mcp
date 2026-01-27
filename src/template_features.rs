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

pub fn handle_template_features(needed: &Features, input: &str) -> Option<String> {
    // what blocks are we currently in
    let mut active_blocks = Features::new();

    let mut modified = false;

    let trailing_newline_count = input.chars().rev().take_while(|&c| c == '\n').count();

    let output_lines = input
        .lines()
        .filter_map(|line| {
            if line.trim_start().starts_with(START_TOKEN) {
                modified = true;
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
        .collect::<Vec<String>>();

    let output = output_lines.join("\n");
    let output = if trailing_newline_count > 0 {
        format!(
            "{}{}",
            output.trim_end_matches('\n'),
            "\n".repeat(trailing_newline_count)
        )
    } else {
        output
    };

    if modified { Some(output) } else { None }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_replacement() {
        const INPUT: &str = r#"
            // random js code
        "#;
        let output = super::handle_template_features(&Features { auth: true }, INPUT);
        assert_eq!(output, None);
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
        assert_eq!(output, Some(EXPECT.to_string()));
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
        assert_eq!(output, Some(EXPECT.to_string()));
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

    #[test]
    fn trailing_newline_single_not_double() {
        const INPUT: &str = r#"
            // START_OF Features.Auth
            // const x = 1;
            // END_OF Features.Auth
        "#;
        const EXPECT: &str = r#"
            const x = 1;
        "#;
        let output = super::handle_template_features(&Features { auth: true }, INPUT);
        assert_eq!(output, Some(EXPECT.to_string()));
    }

    #[test]
    fn trailing_newline_restored_when_last_line_filtered() {
        const INPUT: &str = r#"
            // START_OF Features.Auth
            // x
            // END_OF Features.Auth
        "#;
        const EXPECT: &str = r#"
        "#;
        let output = super::handle_template_features(&Features { auth: false }, INPUT);
        assert_eq!(output, Some(EXPECT.to_string()));
    }

    #[test]
    fn no_extra_newline_when_input_has_no_trailing_newline() {
        const INPUT: &str = r#"
            // START_OF Features.Auth
            // const x = 1;
            // END_OF Features.Auth
        "#;
        const EXPECT: &str = r#"
            const x = 1;
        "#;
        let output = super::handle_template_features(&Features { auth: true }, INPUT);
        assert_eq!(output, Some(EXPECT.to_string()));
    }

    #[test]
    fn trailing_newline_with_multiple_blocks() {
        const INPUT: &str = r#"
            // START_OF Features.Auth
            // const a = 1;
            // END_OF Features.Auth
            outside
            // START_OF Features.Auth
            // const b = 2;
            // END_OF Features.Auth
        "#;
        const EXPECT: &str = r#"
            const a = 1;
            outside
            const b = 2;
        "#;
        let output = super::handle_template_features(&Features { auth: true }, INPUT);
        assert_eq!(output, Some(EXPECT.to_string()));
    }

    #[test]
    fn preserves_double_trailing_newline() {
        const INPUT: &str =
            "// START_OF Features.Auth\n// const x = 1;\n// END_OF Features.Auth\n\n";
        const EXPECT: &str = "const x = 1;\n\n";
        let output = super::handle_template_features(&Features { auth: true }, INPUT);
        assert_eq!(output, Some(EXPECT.to_string()));
    }

    #[test]
    fn preserves_triple_trailing_newline() {
        const INPUT: &str =
            "// START_OF Features.Auth\n// const x = 1;\n// END_OF Features.Auth\n\n\n";
        const EXPECT: &str = "const x = 1;\n\n\n";
        let output = super::handle_template_features(&Features { auth: true }, INPUT);
        assert_eq!(output, Some(EXPECT.to_string()));
    }

    #[test]
    fn preserves_single_trailing_newline() {
        const INPUT: &str = r#"
            // START_OF Features.Auth
            // const x = 1;
            // END_OF Features.Auth
        "#;
        const EXPECT: &str = r#"
            const x = 1;
        "#;
        let output = super::handle_template_features(&Features { auth: true }, INPUT);
        assert_eq!(output, Some(EXPECT.to_string()));
    }

    #[test]
    fn preserves_no_trailing_newline() {
        const INPUT: &str = r#"
            // START_OF Features.Auth
            // const x = 1;
            // END_OF Features.Auth
        "#;
        const EXPECT: &str = r#"
            const x = 1;
        "#;
        let output = super::handle_template_features(&Features { auth: true }, INPUT);
        assert_eq!(output, Some(EXPECT.to_string()));
    }
}
