//! Adapted from <https://github.com/YarnSpinnerTool/YarnSpinner/blob/da39c7195107d8211f21c263e4084f773b84eaff/YarnSpinner.Tests/TestPlan.cs>

use crate::prelude::*;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TestPlan {
    pub next_expected_step: ExpectedStepType,
    pub next_expected_options: Vec<ProcessedOption>,
    pub next_step_value: Option<StepValue>,
    steps: Vec<Step>,
    current_test_plan_step: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessedOption {
    pub line: String,
    pub enabled: bool,
}

impl TestPlan {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn read(path: impl AsRef<Path>) -> Self {
        let steps = fs::read_to_string(path)
            .unwrap()
            .lines()
            // Skip commented lines
            .filter(|line| !line.trim_start().starts_with('#'))
            // Skip empty or blank lines
            .filter(|line| !line.trim().is_empty())
            .map(Step::read)
            .collect();
        Self {
            steps,
            ..Default::default()
        }
    }

    pub fn next(&mut self, dialogue: &mut Dialogue) {
        // step through the test plan until we hit an expectation to
        // see a line, option, or command. specifically, we're waiting
        // to see if we got a Line, Select, Command or Assert step
        // type.
        if self.next_expected_step == ExpectedStepType::Select {
            // our previously-notified task was to select an option.
            // we've now moved past that, so clear the list of expected
            // options.
            self.next_expected_options.clear();
            self.next_step_value = Some(StepValue::Number(0));
        }

        for current_step in self.steps.iter().skip(self.current_test_plan_step) {
            self.current_test_plan_step += 1;

            match current_step.expected_step_type {
                ExpectedStepType::Option => {
                    let Some(StepValue::String(line)) = current_step.value.clone() else {
                        panic!("Expected option line to be a string");
                    };

                    self.next_expected_options.push(ProcessedOption {
                        line,
                        enabled: current_step.expect_option_enabled,
                    });
                }
                ExpectedStepType::Line | ExpectedStepType::Command | ExpectedStepType::Select => {
                    self.next_expected_step = current_step.expected_step_type;
                    self.next_step_value.clone_from(&current_step.value);
                    return;
                }
                ExpectedStepType::Stop => {
                    self.next_expected_step = current_step.expected_step_type;
                    return;
                }
                ExpectedStepType::Set => {
                    let Some(StepValue::StringPair(var, value)) = current_step.value.clone() else {
                        panic!("Expected run line to be a pair of strings");
                    };

                    // .unwrap() used to panic on error as this is used only in Test
                    let current_value = dialogue.variable_storage().get(&var).unwrap();
                    let new_value = match current_value {
                        YarnValue::Number(_) => YarnValue::Number(value.parse::<f32>().unwrap()),
                        YarnValue::String(_) => YarnValue::String(value),
                        YarnValue::Boolean(_) => YarnValue::Boolean(value.parse::<bool>().unwrap()),
                    };

                    println!("INFO: Variable {} set to {}", var, new_value);
                    dialogue.variable_storage_mut().set(var, new_value).unwrap();
                }
                ExpectedStepType::Run => {
                    let Some(StepValue::String(next_node)) = current_step.value.clone() else {
                        panic!("Expected run line to be a string");
                    };

                    println!("INFO: Jumped to node {}", next_node);
                    let _ = dialogue.set_node(next_node);
                }
            }
        }

        // We've fallen off the end of the test plan step list. We
        // expect a stop here.
        self.next_expected_step = ExpectedStepType::Stop;
    }

    pub fn current_step(&self) -> Option<Step> {
        self.steps.get(self.current_test_plan_step).cloned()
    }

    pub fn expect_line(mut self, line: impl Into<String>) -> Self {
        self.steps.push(Step::from_line(line));
        self
    }

    pub fn expect_option(mut self, line: impl Into<String>) -> Self {
        self.steps.push(Step::from_option(line));
        self
    }

    pub fn expect_command(mut self, line: impl Into<String>) -> Self {
        self.steps.push(Step::from_command(line));
        self
    }

    pub fn then_select(mut self, selection: usize) -> Self {
        self.steps.push(Step::from_select(selection));
        self
    }

    pub fn then_set(mut self, variable_name: impl Into<String>, value: impl Into<String>) -> Self {
        self.steps.push(Step::from_set(variable_name, value));
        self
    }

    pub fn then_run(mut self, node_name: impl Into<String>) -> Self {
        self.steps.push(Step::from_run(node_name));
        self
    }

    pub fn expect_stop(mut self) -> Self {
        self.steps.push(Step::from_stop());
        self
    }
}
