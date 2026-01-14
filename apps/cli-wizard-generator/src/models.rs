use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WizardDefinition {
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub questions: Vec<Question>,
    #[serde(default)]
    pub output: Option<OutputConfig>,
    #[serde(default)]
    pub outputs: Vec<OutputConfig>,
}

impl WizardDefinition {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            description: String::new(),
            questions: Vec::new(),
            output: None,
            outputs: Vec::new(),
        }
    }

    pub fn get_outputs(&self) -> Vec<&OutputConfig> {
        if let Some(ref output) = self.output {
            vec![output]
        } else {
            self.outputs.iter().collect()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Question {
    pub id: String,
    #[serde(rename = "type")]
    pub question_type: QuestionType,
    pub prompt: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub default: Option<serde_json::Value>,
    #[serde(default)]
    pub validation: Option<Validation>,
    #[serde(default)]
    pub options: Vec<QuestionOption>,
    #[serde(default)]
    pub when: Option<String>,
}

impl Question {
    pub fn new(id: &str, question_type: QuestionType, prompt: &str) -> Self {
        Self {
            id: id.to_string(),
            question_type,
            prompt: prompt.to_string(),
            description: None,
            default: None,
            validation: None,
            options: Vec::new(),
            when: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QuestionType {
    Text,
    Password,
    Number,
    Select,
    MultiSelect,
    Confirm,
    Path,
}

impl QuestionType {
    pub fn as_str(&self) -> &str {
        match self {
            QuestionType::Text => "text",
            QuestionType::Password => "password",
            QuestionType::Number => "number",
            QuestionType::Select => "select",
            QuestionType::MultiSelect => "multi_select",
            QuestionType::Confirm => "confirm",
            QuestionType::Path => "path",
        }
    }

    pub fn icon(&self) -> &str {
        match self {
            QuestionType::Text => "📝",
            QuestionType::Password => "🔒",
            QuestionType::Number => "🔢",
            QuestionType::Select => "📋",
            QuestionType::MultiSelect => "☑️",
            QuestionType::Confirm => "❓",
            QuestionType::Path => "📁",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionOption {
    pub label: String,
    pub value: String,
    #[serde(default)]
    pub description: Option<String>,
}

impl QuestionOption {
    pub fn new(label: &str, value: &str) -> Self {
        Self {
            label: label.to_string(),
            value: value.to_string(),
            description: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Validation {
    #[serde(default)]
    pub pattern: Option<String>,
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default)]
    pub min: Option<f64>,
    #[serde(default)]
    pub max: Option<f64>,
    #[serde(default)]
    pub min_length: Option<usize>,
    #[serde(default)]
    pub max_length: Option<usize>,
    #[serde(default)]
    pub required: bool,
}

impl Default for Validation {
    fn default() -> Self {
        Self {
            pattern: None,
            message: None,
            min: None,
            max: None,
            min_length: None,
            max_length: None,
            required: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    #[serde(rename = "type")]
    pub output_type: OutputType,
    #[serde(default)]
    pub template: Option<String>,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub shell: Option<String>,
    #[serde(default)]
    pub shells: Vec<String>,
    #[serde(default)]
    pub templates: std::collections::HashMap<String, String>,
}

impl OutputConfig {
    pub fn new(output_type: OutputType) -> Self {
        Self {
            output_type,
            template: None,
            path: None,
            shell: None,
            shells: Vec::new(),
            templates: std::collections::HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputType {
    File,
    Script,
    Wizard,
    Stdout,
}

impl OutputType {
    pub fn as_str(&self) -> &str {
        match self {
            OutputType::File => "file",
            OutputType::Script => "script",
            OutputType::Wizard => "wizard",
            OutputType::Stdout => "stdout",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Answer {
    pub question_id: String,
    pub value: AnswerValue,
    pub timestamp: DateTime<Utc>,
}

impl Answer {
    pub fn new(question_id: &str, value: AnswerValue) -> Self {
        Self {
            question_id: question_id.to_string(),
            value,
            timestamp: Utc::now(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum AnswerValue {
    Text(String),
    Number(f64),
    Boolean(bool),
    Selected(String),
    MultiSelected(Vec<String>),
}

impl AnswerValue {
    pub fn as_json(&self) -> serde_json::Value {
        match self {
            AnswerValue::Text(s) => serde_json::Value::String(s.clone()),
            AnswerValue::Number(n) => serde_json::json!(n),
            AnswerValue::Boolean(b) => serde_json::Value::Bool(*b),
            AnswerValue::Selected(s) => serde_json::Value::String(s.clone()),
            AnswerValue::MultiSelected(v) => serde_json::json!(v),
        }
    }
}

#[derive(Debug, Clone)]
pub struct WizardSession {
    pub id: Uuid,
    pub wizard: WizardDefinition,
    pub answers: Vec<Answer>,
    pub current_question: usize,
    pub started_at: DateTime<Utc>,
    pub completed: bool,
}

impl WizardSession {
    pub fn new(wizard: WizardDefinition) -> Self {
        Self {
            id: Uuid::new_v4(),
            wizard,
            answers: Vec::new(),
            current_question: 0,
            started_at: Utc::now(),
            completed: false,
        }
    }

    pub fn current_question(&self) -> Option<&Question> {
        self.wizard.questions.get(self.current_question)
    }

    pub fn progress(&self) -> (usize, usize) {
        (self.current_question + 1, self.wizard.questions.len())
    }

    pub fn get_answer(&self, question_id: &str) -> Option<&Answer> {
        self.answers.iter().find(|a| a.question_id == question_id)
    }

    pub fn get_context(&self) -> serde_json::Map<String, serde_json::Value> {
        let mut context = serde_json::Map::new();
        for answer in &self.answers {
            context.insert(answer.question_id.clone(), answer.value.as_json());
        }
        context
    }

    pub fn should_show_question(&self, question: &Question) -> bool {
        if let Some(ref condition) = question.when {
            self.evaluate_condition(condition)
        } else {
            true
        }
    }

    fn evaluate_condition(&self, condition: &str) -> bool {
        // Simple condition evaluation: "var_name" (truthy check) or "var_name == value"
        if condition.contains("==") {
            let parts: Vec<&str> = condition.split("==").map(|s| s.trim()).collect();
            if parts.len() == 2 {
                let var_name = parts[0];
                let expected = parts[1].trim_matches(|c| c == '"' || c == '\'');

                if let Some(answer) = self.get_answer(var_name) {
                    return match &answer.value {
                        AnswerValue::Text(s) => s == expected,
                        AnswerValue::Boolean(b) => {
                            let expected_bool = expected == "true" || expected == "yes";
                            *b == expected_bool
                        }
                        AnswerValue::Selected(s) => s == expected,
                        _ => false,
                    };
                }
            }
        } else if condition.contains("contains") {
            let parts: Vec<&str> = condition.split("contains").map(|s| s.trim()).collect();
            if parts.len() == 2 {
                let var_name = parts[0];
                let search_value = parts[1].trim_matches(|c| c == '"' || c == '\'' || c == ' ');

                if let Some(answer) = self.get_answer(var_name) {
                    return match &answer.value {
                        AnswerValue::MultiSelected(v) => v.iter().any(|s| s == search_value),
                        AnswerValue::Text(s) => s.contains(search_value),
                        _ => false,
                    };
                }
            }
        } else {
            // Simple truthy check
            let var_name = condition.trim();
            if let Some(answer) = self.get_answer(var_name) {
                return match &answer.value {
                    AnswerValue::Boolean(b) => *b,
                    AnswerValue::Text(s) => !s.is_empty(),
                    AnswerValue::Selected(s) => !s.is_empty(),
                    AnswerValue::MultiSelected(v) => !v.is_empty(),
                    AnswerValue::Number(_) => true,
                };
            }
        }

        false
    }
}

#[derive(Debug, Clone)]
pub struct GeneratedOutput {
    pub path: Option<String>,
    pub content: String,
    pub output_type: OutputType,
}

#[derive(Debug, Clone)]
pub struct ValidationError {
    pub message: String,
}

impl ValidationError {
    pub fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
        }
    }
}
