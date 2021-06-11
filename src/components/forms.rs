use validator::ValidationErrors;

pub fn filter_errors(name: &str, errors: &ValidationErrors) -> Vec<String> {
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

// Taken from
// https://github.com/utkarshkukreti/markup.rs/blob/master/markup/src/escape.rs
pub fn escape(str: &str, w: &mut impl std::fmt::Write) -> std::fmt::Result {
    let mut last = 0;
    for (index, byte) in str.bytes().enumerate() {
        match byte {
            b'&' | b'<' | b'>' | b'"' => {
                w.write_str(&str[last..index])?;
                w.write_str(match byte {
                    b'&' => "&amp;",
                    b'<' => "&lt;",
                    b'>' => "&gt;",
                    _ => "&quot;",
                })?;
                last = index + 1;
            }
            _ => {}
        }
    }
    w.write_str(&str[last..])
}

#[derive(Debug)]
pub enum InputType {
    Text,
    //Date,
    Password,
}

impl Default for InputType {
    fn default() -> Self {
        InputType::Text
    }
}

#[derive(Default)]
pub struct Help {
    pub placeholder: String,
    pub help_text: String,
}

pub struct Stimulus {
    pub data_target: Option<String>,
    pub data_action: Option<String>,
}

#[derive(Default)]
pub struct FormInput {
    pub input_type: InputType,
    pub name: String,
    pub value: String,
    pub label: String,
    pub stimulus: Option<Stimulus>,
    pub help: Option<Help>,
    pub errors: Option<ValidationErrors>,
}

impl markup::Render for FormInput {
    fn write_to(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
        write!(f, "<label for='{}'>{}</label>", self.name, self.label)?;

        dbg!(&self.errors);

        let input_type = format!("{:?}", self.input_type);

        if self.errors.is_some() {
            let mut val = String::new();
            escape(&self.value, &mut val)?;
            write!(
                f,
                "<input class='error', id='{}' name='{}' type={:?} value='{}'",
                self.name,
                self.name,
                input_type.to_lowercase(),
                val
            )?;
        } else {
            let mut val = String::new();
            escape(&self.value, &mut val)?;
            write!(
                f,
                "<input id='{}' name='{}' type={:?} value='{}'",
                self.name,
                self.name,
                input_type.to_lowercase(),
                val
            )?;
        }

        // data-target='form.email' data-action='form.click'
        if let Some(stimulus) = &self.stimulus {
            if let Some(data_target) = &stimulus.data_target {
                write!(f, " data-target='{}'", data_target)?;
            }
            if let Some(data_action) = &stimulus.data_action {
                write!(f, " data-action='{}'", data_action)?;
            }
        }

        if let Some(help) = &self.help {
            write!(f, " placeholder='{}' />", help.placeholder)?;
        } else {
            write!(f, "/>")?;
        }

        if let Some(errors) = &self.errors {
            for error in &filter_errors(&self.name, errors) {
                write!(f, "<span class='error'>{}</span>", error)?;
            }
        }

        if let Some(help) = &self.help {
            write!(f, "<span class='a_help_text'>{}</span>", help.help_text)?;
        }

        Ok(())
    }
}

markup::define! {
    PlaceholderInput<'a>(title: &'a str, name: &'a str, value: &'a str, input_type: &'a str,
        help_text: &'a str, placeholder: &'a str, errors: &'a ValidationErrors) {
        label[for=name] { {title} }
        @if !filter_errors(name, errors).is_empty() {
            input.error[id=name, name=name, placeholder=placeholder, value=value, type=input_type] {}
        } else {
            input[id=name, name=name, placeholder=placeholder, value=value, type=input_type] {}
        }
        @for error in &filter_errors(name, errors) {
            span.error {
                {error}
            }
        }
        span.a_help_text { {help_text} }
    }
    Input<'a>(title: &'a str, name: &'a str, value: &'a str, input_type: &'a str,
        help_text: &'a str, errors: &'a ValidationErrors) {
        { PlaceholderInput{ title, name, value, input_type, help_text, placeholder: "", errors} }
    }
    EmailInput<'a>(title: &'a str, name: &'a str, value: &'a str, help_text: &'a str, errors: &'a ValidationErrors) {
        { Input{ title, name, value, input_type: "email", help_text, errors } }
    }
    NumberInput<'a>(title: &'a str, name: &'a str, value: &'a str, help_text: &'a str, errors: &'a ValidationErrors) {
        { Input{ title, name, value, input_type: "number", help_text, errors } }
    }
    TextInput<'a>(title: &'a str, name: &'a str, value: &'a str, help_text: &'a str, errors: &'a ValidationErrors) {
        { Input{ title, name, value, input_type: "text", help_text, errors } }
    }
    PasswordInput<'a>(title: &'a str, name: &'a str, value: &'a str, help_text: &'a str, errors: &'a ValidationErrors) {
        { Input{ title, name, value, input_type: "password", help_text, errors } }
    }
    DateInput<'a>(title: &'a str, name: &'a str, value: &'a str, help_text: &'a str, errors: &'a ValidationErrors) {
        { Input{ title, name, value, input_type: "date", help_text, errors } }
    }
    SearchInput<'a>(title: &'a str, name: &'a str, value: &'a str, help_text: &'a str, placeholder: &'a str) {
        { PlaceholderInput{ title, name, value, input_type: "text",
            help_text, placeholder, errors: &ValidationErrors::default() } }
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
