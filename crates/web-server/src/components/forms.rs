use std::fmt::Write;
use validator::ValidationErrors;

fn filter_errors(name: &str, errors: &ValidationErrors) -> Vec<String> {
    let mut filtered_errors: Vec<String> = Default::default();
    if let Some(errs) = errors.field_errors().get(name) {
        for error in errs.iter() {
            if let Some(err) = &error.message {
                filtered_errors.push(err.to_string());
            }
        }
    }
    filtered_errors
}

pub fn _escape(src: &str) -> String {
    let mut escaped = String::with_capacity(src.len());
    let mut utf16_buf = [0u16; 2];
    for c in src.chars() {
        match c {
            '&' => escaped += "",
            '<' => escaped += "",
            '>' => escaped += "",
            '"' => escaped += "",
            c => {
                let encoded = c.encode_utf16(&mut utf16_buf);
                for utf16 in encoded {
                    write!(&mut escaped, "\\u{:04X}", utf16).unwrap();
                }
            }
        }
    }
    escaped
}

markup::define! {
    PlaceholderInput<'a>(title: &'a str, name: &'a str, value: &'a str, input_type: &'a str,
        help_text: &'a str, placeholder: &'a str, autocomplete: &'a str, errors: &'a ValidationErrors) {
        label[for=name] { {title} }
        @if !filter_errors(name, errors).is_empty() {
            input.error[id=name, name=name, placeholder=placeholder, autocomplete=autocomplete, value=value, type=input_type] {}
        } else {
            input[id=name, name=name, placeholder=placeholder, autocomplete=autocomplete, value=value, type=input_type] {}
        }
        @for error in &filter_errors(name, errors) {
            span.error {
                {error}
            }
        }
        span.a_help_text { {help_text} }
    }
    Input<'a>(title: &'a str, name: &'a str, value: &'a str, input_type: &'a str, autocomplete: &'a str,
        help_text: &'a str, errors: &'a ValidationErrors) {
        { PlaceholderInput{ title, name, value, input_type, help_text, placeholder: "", autocomplete, errors} }
    }
    EmailInput<'a>(title: &'a str, name: &'a str, value: &'a str, autocomplete: &'a str, help_text: &'a str, errors: &'a ValidationErrors) {
        { Input{ title, name, value, input_type: "email", autocomplete, help_text, errors } }
    }
    TextInput<'a>(title: &'a str, name: &'a str, value: &'a str, help_text: &'a str, errors: &'a ValidationErrors) {
        { Input{ title, name, value, input_type: "text", autocomplete: "", help_text, errors } }
    }
    PasswordInput<'a>(title: &'a str, name: &'a str, value: &'a str, autocomplete: &'a str, help_text: &'a str, errors: &'a ValidationErrors) {
        { Input{ title, name, value, input_type: "password", autocomplete, help_text, errors } }
    }
    DateInput<'a>(title: &'a str, name: &'a str, value: &'a str, help_text: &'a str, errors: &'a ValidationErrors) {
        { Input{ title, name, value, input_type: "date", autocomplete: "", help_text, errors } }
    }
    SearchInput<'a>(title: &'a str, name: &'a str, value: &'a str, help_text: &'a str, placeholder: &'a str) {
        { PlaceholderInput{ title, name, value, input_type: "text",
            help_text, placeholder, autocomplete: "", errors: &ValidationErrors::default() } }
    }

    TextArea<'a>(title: &'a str, name: &'a str, value: &'a str, help_text: &'a str, placeholder: &'a str, errors: &'a ValidationErrors) {
        label[for=name] { {title} }
        @if !filter_errors(name, errors).is_empty() {
            textarea.error[id=name, name=name, placeholder=placeholder] { {value} }
        } else {
            textarea[id=name, name=name, placeholder=placeholder]  { {value} }
        }
        @for error in &filter_errors(name, errors) {
            span.error {
                {error}
            }
        }
        span.a_help_text { {help_text} }
    }
}
