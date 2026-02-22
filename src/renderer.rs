use crate::arazzo::{Action, ActionType, ArazzoDocument, Step, Workflow};

pub trait Renderer {
    fn render(&self, document: &ArazzoDocument) -> String;
}

pub struct MermaidFlowchart;

impl Renderer for MermaidFlowchart {
    fn render(&self, arazzo: &ArazzoDocument) -> String {
        let mut output = title(arazzo);
        output.push_str("flowchart TD\n");

        for workflow in &arazzo.workflows {
            output.push_str(&subgraph(workflow));

            for (i, current_step) in workflow.steps.iter().enumerate() {
                if has_goto_actions(current_step) {
                    output.push_str(&to_rhombus_from_rectangle(current_step));

                    for verdict in [Verdict::Ok, Verdict::Ng] {
                        if let Some(actions) = lookup_goto_actions(current_step, verdict) {
                            for action in actions {
                                if let Some(action_step_id) = &action.step_id {
                                    output.push_str(&to_rectangle_from_rhombus(
                                        current_step,
                                        verdict,
                                        action_step_id,
                                    ));
                                }
                            }
                        }
                    }
                } else if let Some(next_step) = &workflow.steps.get(i + 1) {
                    output.push_str(&to_rectangle_from_rectangle(current_step, next_step));
                }
            }

            output.push_str("    end\n");
        }

        output
    }
}

fn has_goto_actions(step: &Step) -> bool {
    lookup_goto_actions(step, Verdict::Ok).is_some_and(|a| !a.is_empty())
        || lookup_goto_actions(step, Verdict::Ng).is_some_and(|a| !a.is_empty())
}

fn lookup_goto_actions(step: &Step, verdict: Verdict) -> Option<Vec<&Action>> {
    let actions = match verdict {
        Verdict::Ok => step.on_success.as_ref(),
        Verdict::Ng => step.on_failure.as_ref(),
    };

    actions.map(|vec| {
        vec.iter()
            .filter(|a| a.action_type == ActionType::Goto)
            .collect::<Vec<_>>()
    })
}

fn to_rhombus_from_rectangle(step: &Step) -> String {
    format!(
        "    {}{} --> {}{}\n",
        step.step_id,
        node_label(step),
        middle_node(step),
        condition(step),
    )
}

fn to_rectangle_from_rhombus(step: &Step, verdict: Verdict, step_id: &str) -> String {
    format!(
        "    {}{} -->|{}| {}\n",
        middle_node(step),
        condition(step),
        match verdict {
            Verdict::Ok => "true",
            Verdict::Ng => "false",
        },
        step_id,
    )
}

fn to_rectangle_from_rectangle(from: &Step, to: &Step) -> String {
    format!(
        "    {}{} --> {}{}\n",
        from.step_id,
        node_label(from),
        to.step_id,
        node_label(to),
    )
}

#[derive(Clone, Copy)]
enum Verdict {
    Ok,
    Ng,
}

fn title(arazzo: &ArazzoDocument) -> String {
    format!("---\ntitle: {}\n---\n", arazzo.info.title)
}

fn subgraph(workflow: &Workflow) -> String {
    format!(
        "    subgraph {}{}\n",
        workflow.workflow_id,
        workflow
            .description
            .as_ref()
            .map_or(String::from(""), |v| format!("[\"{}\"]", v))
    )
}

fn condition(step: &Step) -> String {
    step.success_criteria
        .as_ref()
        .and_then(|cs| cs.first())
        .and_then(|c| c.condition.as_ref())
        .map_or(String::from(""), |v| format!("{{{}}}", v))
}

fn node_label(step: &Step) -> String {
    step.description
        .as_ref()
        .map_or(String::from(""), |v| format!("[\"{}\"]", v))
}

fn middle_node(step: &Step) -> String {
    format!("{}Node", step.step_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arazzo::{Criteria, Info};

    #[test]
    fn render_steps() {
        let arazzo = ArazzoDocument {
            info: Info {
                title: String::from("Workflows"),
            },
            workflows: vec![Workflow {
                workflow_id: String::from("workflowFoo"),
                description: Some(String::from("Workflow foo's description.")),
                steps: vec![
                    Step {
                        step_id: String::from("stepFoo"),
                        description: Some(String::from("Step foo's description.")),
                        success_criteria: Some(vec![Criteria {
                            condition: Some(String::from("$statusCode == 200")),
                        }]),
                        on_success: Some(vec![Action {
                            action_type: ActionType::Goto,
                            step_id: Some(String::from("stepBar")),
                        }]),
                        on_failure: Some(vec![Action {
                            action_type: ActionType::Goto,
                            step_id: Some(String::from("stepBaz")),
                        }]),
                    },
                    Step {
                        step_id: String::from("stepBar"),
                        description: Some(String::from("Step bar's description.")),
                        success_criteria: Some(vec![Criteria {
                            condition: Some(String::from("$statusCode == 200")),
                        }]),
                        on_success: Some(vec![Action {
                            action_type: ActionType::End,
                            step_id: None,
                        }]),
                        on_failure: Some(vec![Action {
                            action_type: ActionType::Goto,
                            step_id: Some(String::from("stepBaz")),
                        }]),
                    },
                    Step {
                        step_id: String::from("step_baz"),
                        description: Some(String::from("description_baz")),
                        success_criteria: Some(vec![Criteria {
                            condition: Some(String::from("$statusCode == 200")),
                        }]),
                        on_success: Some(vec![Action {
                            action_type: ActionType::End,
                            step_id: None,
                        }]),
                        on_failure: Some(vec![Action {
                            action_type: ActionType::End,
                            step_id: None,
                        }]),
                    },
                ],
            }],
        };

        let sut = MermaidFlowchart;

        let actual = sut.render(&arazzo);

        let expected = concat!(
            "---\n",
            "title: Workflows\n",
            "---\n",
            "flowchart TD\n",
            "    subgraph workflowFoo[\"Workflow foo's description.\"]\n",
            "    stepFoo[\"Step foo's description.\"] --> stepFooNode{$statusCode == 200}\n",
            "    stepFooNode{$statusCode == 200} -->|true| stepBar\n",
            "    stepFooNode{$statusCode == 200} -->|false| stepBaz\n",
            "    stepBar[\"Step bar's description.\"] --> stepBarNode{$statusCode == 200}\n",
            "    stepBarNode{$statusCode == 200} -->|false| stepBaz\n",
            "    end\n",
        );

        assert_eq!(expected, actual);
    }

    #[test]
    fn render_steps_without_description() {
        let arazzo = ArazzoDocument {
            info: Info {
                title: String::from("workflows"),
            },
            workflows: vec![Workflow {
                workflow_id: String::from("workflow_foo"),
                description: None,
                steps: vec![
                    Step {
                        step_id: String::from("step_foo"),
                        description: None,
                        success_criteria: None,
                        on_success: None,
                        on_failure: None,
                    },
                    Step {
                        step_id: String::from("step_bar"),
                        description: None,
                        success_criteria: None,
                        on_success: None,
                        on_failure: None,
                    },
                ],
            }],
        };

        let sut = MermaidFlowchart;

        let actual = sut.render(&arazzo);

        let expected = concat!(
            "---\n",
            "title: workflows\n",
            "---\n",
            "flowchart TD\n",
            "    subgraph workflow_foo\n",
            "    step_foo --> step_bar\n",
            "    end\n",
        );

        assert_eq!(expected, actual);
    }
}
