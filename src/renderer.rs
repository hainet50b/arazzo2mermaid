use crate::arazzo::{ActionType, ArazzoDocument, Step, Workflow};

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
                if should_branch(current_step) {
                    output.push_str(&to_rhombus_from_rectangle(current_step));

                    if let Some(actions) = &current_step.on_success {
                        if let Some(action) = actions.first() {
                            match action.action_type {
                                ActionType::Goto => {
                                    if let Some(action_step_id) = &action.step_id {
                                        output.push_str(&to_rectangle_from_rhombus(
                                            current_step,
                                            Verdict::Ok,
                                            action_step_id,
                                        ))
                                    }
                                }
                                ActionType::End => {
                                    output.push_str(&to_end_from_rhombus(
                                        current_step,
                                        Verdict::Ok,
                                        workflow,
                                    ));
                                }
                            }
                        }
                    } else if let Some(next_step) = &workflow.steps.get(i + 1) {
                        output.push_str(&to_rectangle_from_rhombus(
                            current_step,
                            Verdict::Ok,
                            &next_step.step_id,
                        ));
                    } else {
                        output.push_str(&to_end_from_rhombus(current_step, Verdict::Ok, workflow));
                    }

                    if let Some(actions) = &current_step.on_failure {
                        if let Some(action) = actions.first() {
                            match action.action_type {
                                ActionType::Goto => {
                                    if let Some(action_step_id) = &action.step_id {
                                        output.push_str(&to_rectangle_from_rhombus(
                                            current_step,
                                            Verdict::Ng,
                                            action_step_id,
                                        ))
                                    }
                                }
                                ActionType::End => {
                                    output.push_str(&to_end_from_rhombus(
                                        current_step,
                                        Verdict::Ng,
                                        workflow,
                                    ));
                                }
                            }
                        }
                    } else {
                        output.push_str(&to_end_from_rhombus(current_step, Verdict::Ng, workflow));
                    }
                } else if let Some(next_step) = &workflow.steps.get(i + 1) {
                    output.push_str(&to_rectangle_from_rectangle(current_step, next_step));
                } else {
                    output.push_str(&to_end_from_rectangle(current_step, workflow));
                }
            }

            output.push_str("    end\n");
        }

        output
    }
}

fn should_branch(step: &Step) -> bool {
    step.success_criteria.is_some() || step.on_success.is_some() || step.on_failure.is_some()
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
            .map_or(String::new(), |v| format!("[\"{}\"]", v))
    )
}

fn to_rhombus_from_rectangle(step: &Step) -> String {
    format!(
        "    {rectangle_node} --> {rhombus_node}\n",
        rectangle_node = rectangle_node(step),
        rhombus_node = rhombus_node(step),
    )
}

fn to_rectangle_from_rhombus(step: &Step, verdict: Verdict, step_id: &str) -> String {
    format!(
        "    {rhombus_node} -->|{verdict}| {rectangle_node}\n",
        rhombus_node = rhombus_node(step),
        verdict = match verdict {
            Verdict::Ok => "true",
            Verdict::Ng => "false",
        },
        rectangle_node = step_id,
    )
}

fn to_rectangle_from_rectangle(from: &Step, to: &Step) -> String {
    format!(
        "    {from_node} --> {to_node}\n",
        from_node = rectangle_node(from),
        to_node = rectangle_node(to),
    )
}

fn to_end_from_rectangle(step: &Step, workflow: &Workflow) -> String {
    format!(
        "    {rectangle_node} --> {end_node}\n",
        rectangle_node = rectangle_node(step),
        end_node = end_node(workflow),
    )
}

fn to_end_from_rhombus(step: &Step, verdict: Verdict, workflow: &Workflow) -> String {
    format!(
        "    {rhombus_node} -->|{verdict}| {end_node}\n",
        rhombus_node = rhombus_node(step),
        verdict = match verdict {
            Verdict::Ok => "true",
            Verdict::Ng => "false",
        },
        end_node = end_node(workflow),
    )
}

#[derive(Clone, Copy)]
enum Verdict {
    Ok,
    Ng,
}

fn rectangle_node(step: &Step) -> String {
    format!(
        "{node_name}{node_label}",
        node_name = step.step_id,
        node_label = node_label(step),
    )
}

fn node_label(step: &Step) -> String {
    step.description
        .as_ref()
        .map_or(String::new(), |v| format!("[\"{}\"]", v))
}

fn rhombus_node(step: &Step) -> String {
    format!(
        "{node_name}Node{condition}",
        node_name = step.step_id,
        condition = condition(step),
    )
}

fn condition(step: &Step) -> String {
    if let Some(criteria) = &step.success_criteria
        && !criteria.is_empty()
    {
        let condition = criteria
            .iter()
            .filter_map(|c| c.condition.as_deref())
            .collect::<Vec<&str>>()
            .join(" && ");

        if condition.is_empty() {
            String::new()
        } else {
            format!("{{{condition}}}")
        }
    } else {
        String::new()
    }
}

fn end_node(workflow: &Workflow) -> String {
    format!("{}EndNode((End))", workflow.workflow_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arazzo::{Action, Criteria, Info};

    #[test]
    fn render_full() {
        let arazzo = ArazzoDocument {
            info: Info {
                title: String::from("Workflows"),
            },
            workflows: vec![
                Workflow {
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
                            step_id: String::from("stepBaz"),
                            description: Some(String::from("Step baz's description.")),
                            success_criteria: None,
                            on_success: None,
                            on_failure: None,
                        },
                    ],
                },
                Workflow {
                    workflow_id: String::from("workflowBar"),
                    description: Some(String::from("Workflow bar's description.")),
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
                            step_id: String::from("stepBaz"),
                            description: Some(String::from("Step baz's description.")),
                            success_criteria: None,
                            on_success: None,
                            on_failure: None,
                        },
                    ],
                },
            ],
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
            "    stepBarNode{$statusCode == 200} -->|true| workflowFooEndNode((End))\n",
            "    stepBarNode{$statusCode == 200} -->|false| stepBaz\n",
            "    stepBaz[\"Step baz's description.\"] --> workflowFooEndNode((End))\n",
            "    end\n",
            "    subgraph workflowBar[\"Workflow bar's description.\"]\n",
            "    stepFoo[\"Step foo's description.\"] --> stepFooNode{$statusCode == 200}\n",
            "    stepFooNode{$statusCode == 200} -->|true| stepBar\n",
            "    stepFooNode{$statusCode == 200} -->|false| stepBaz\n",
            "    stepBar[\"Step bar's description.\"] --> stepBarNode{$statusCode == 200}\n",
            "    stepBarNode{$statusCode == 200} -->|true| workflowBarEndNode((End))\n",
            "    stepBarNode{$statusCode == 200} -->|false| stepBaz\n",
            "    stepBaz[\"Step baz's description.\"] --> workflowBarEndNode((End))\n",
            "    end\n",
        );

        assert_eq!(expected, actual);
    }

    #[test]
    fn render_multiple_workflows() {
        let arazzo = ArazzoDocument {
            info: Info {
                title: String::from("Workflows"),
            },
            workflows: vec![
                Workflow {
                    workflow_id: String::from("workflowFoo"),
                    description: None,
                    steps: vec![Step {
                        step_id: String::from("stepFoo"),
                        description: None,
                        success_criteria: None,
                        on_success: None,
                        on_failure: None,
                    }],
                },
                Workflow {
                    workflow_id: String::from("workflowBar"),
                    description: None,
                    steps: vec![Step {
                        step_id: String::from("stepFoo"),
                        description: None,
                        success_criteria: None,
                        on_success: None,
                        on_failure: None,
                    }],
                },
            ],
        };

        let sut = MermaidFlowchart;

        let actual = sut.render(&arazzo);

        let expected = concat!(
            "---\n",
            "title: Workflows\n",
            "---\n",
            "flowchart TD\n",
            "    subgraph workflowFoo\n",
            "    stepFoo --> workflowFooEndNode((End))\n",
            "    end\n",
            "    subgraph workflowBar\n",
            "    stepFoo --> workflowBarEndNode((End))\n",
            "    end\n",
        );

        assert_eq!(expected, actual);
    }

    #[test]
    fn render_multiple_steps() {
        let arazzo = ArazzoDocument {
            info: Info {
                title: String::from("Workflows"),
            },
            workflows: vec![Workflow {
                workflow_id: String::from("workflowFoo"),
                description: None,
                steps: vec![
                    Step {
                        step_id: String::from("stepFoo"),
                        description: None,
                        success_criteria: None,
                        on_success: None,
                        on_failure: None,
                    },
                    Step {
                        step_id: String::from("stepBar"),
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
            "title: Workflows\n",
            "---\n",
            "flowchart TD\n",
            "    subgraph workflowFoo\n",
            "    stepFoo --> stepBar\n",
            "    stepBar --> workflowFooEndNode((End))\n",
            "    end\n",
        );

        assert_eq!(expected, actual);
    }

    #[test]
    fn render_minimal() {
        let arazzo = ArazzoDocument {
            info: Info {
                title: String::from("Workflows"),
            },
            workflows: vec![Workflow {
                workflow_id: String::from("workflowFoo"),
                description: None,
                steps: vec![Step {
                    step_id: String::from("stepFoo"),
                    description: None,
                    success_criteria: None,
                    on_success: None,
                    on_failure: None,
                }],
            }],
        };

        let sut = MermaidFlowchart;

        let actual = sut.render(&arazzo);

        let expected = concat!(
            "---\n",
            "title: Workflows\n",
            "---\n",
            "flowchart TD\n",
            "    subgraph workflowFoo\n",
            "    stepFoo --> workflowFooEndNode((End))\n",
            "    end\n",
        );

        assert_eq!(expected, actual);
    }

    #[test]
    fn render_success_criteria_defined_on_success_defined_on_failure_defined() {
        let arazzo = ArazzoDocument {
            info: Info {
                title: String::from("Workflows"),
            },
            workflows: vec![Workflow {
                workflow_id: String::from("workflowFoo"),
                description: None,
                steps: vec![
                    Step {
                        step_id: String::from("stepFoo"),
                        description: None,
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
                        description: None,
                        success_criteria: None,
                        on_success: None,
                        on_failure: None,
                    },
                    Step {
                        step_id: String::from("stepBaz"),
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
            "title: Workflows\n",
            "---\n",
            "flowchart TD\n",
            "    subgraph workflowFoo\n",
            "    stepFoo --> stepFooNode{$statusCode == 200}\n",
            "    stepFooNode{$statusCode == 200} -->|true| stepBar\n",
            "    stepFooNode{$statusCode == 200} -->|false| stepBaz\n",
            "    stepBar --> stepBaz\n",
            "    stepBaz --> workflowFooEndNode((End))\n",
            "    end\n",
        );

        assert_eq!(expected, actual);
    }

    #[test]
    fn render_success_criteria_defined_on_success_defined_on_failure_omitted() {
        let arazzo = ArazzoDocument {
            info: Info {
                title: String::from("Workflows"),
            },
            workflows: vec![Workflow {
                workflow_id: String::from("workflowFoo"),
                description: None,
                steps: vec![
                    Step {
                        step_id: String::from("stepFoo"),
                        description: None,
                        success_criteria: Some(vec![Criteria {
                            condition: Some(String::from("$statusCode == 200")),
                        }]),
                        on_success: Some(vec![Action {
                            action_type: ActionType::Goto,
                            step_id: Some(String::from("stepBar")),
                        }]),
                        on_failure: None,
                    },
                    Step {
                        step_id: String::from("stepBar"),
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
            "title: Workflows\n",
            "---\n",
            "flowchart TD\n",
            "    subgraph workflowFoo\n",
            "    stepFoo --> stepFooNode{$statusCode == 200}\n",
            "    stepFooNode{$statusCode == 200} -->|true| stepBar\n",
            "    stepFooNode{$statusCode == 200} -->|false| workflowFooEndNode((End))\n",
            "    stepBar --> workflowFooEndNode((End))\n",
            "    end\n",
        );

        assert_eq!(expected, actual);
    }

    #[test]
    fn render_success_criteria_defined_on_success_omitted_on_failure_defined() {
        let arazzo = ArazzoDocument {
            info: Info {
                title: String::from("Workflows"),
            },
            workflows: vec![Workflow {
                workflow_id: String::from("workflowFoo"),
                description: None,
                steps: vec![
                    Step {
                        step_id: String::from("stepFoo"),
                        description: None,
                        success_criteria: Some(vec![Criteria {
                            condition: Some(String::from("$statusCode == 200")),
                        }]),
                        on_success: None,
                        on_failure: Some(vec![Action {
                            action_type: ActionType::Goto,
                            step_id: Some(String::from("stepBaz")),
                        }]),
                    },
                    Step {
                        step_id: String::from("stepBar"),
                        description: None,
                        success_criteria: None,
                        on_success: None,
                        on_failure: None,
                    },
                    Step {
                        step_id: String::from("stepBaz"),
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
            "title: Workflows\n",
            "---\n",
            "flowchart TD\n",
            "    subgraph workflowFoo\n",
            "    stepFoo --> stepFooNode{$statusCode == 200}\n",
            "    stepFooNode{$statusCode == 200} -->|true| stepBar\n",
            "    stepFooNode{$statusCode == 200} -->|false| stepBaz\n",
            "    stepBar --> stepBaz\n",
            "    stepBaz --> workflowFooEndNode((End))\n",
            "    end\n",
        );

        assert_eq!(expected, actual);
    }

    #[test]
    fn render_success_criteria_defined_on_success_omitted_on_failure_omitted() {
        let arazzo = ArazzoDocument {
            info: Info {
                title: String::from("Workflows"),
            },
            workflows: vec![Workflow {
                workflow_id: String::from("workflowFoo"),
                description: None,
                steps: vec![
                    Step {
                        step_id: String::from("stepFoo"),
                        description: None,
                        success_criteria: Some(vec![Criteria {
                            condition: Some(String::from("$statusCode == 200")),
                        }]),
                        on_success: None,
                        on_failure: None,
                    },
                    Step {
                        step_id: String::from("stepBar"),
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
            "title: Workflows\n",
            "---\n",
            "flowchart TD\n",
            "    subgraph workflowFoo\n",
            "    stepFoo --> stepFooNode{$statusCode == 200}\n",
            "    stepFooNode{$statusCode == 200} -->|true| stepBar\n",
            "    stepFooNode{$statusCode == 200} -->|false| workflowFooEndNode((End))\n",
            "    stepBar --> workflowFooEndNode((End))\n",
            "    end\n",
        );

        assert_eq!(expected, actual);
    }

    #[test]
    fn render_success_criteria_omitted_on_success_defined_on_failure_defined() {
        let arazzo = ArazzoDocument {
            info: Info {
                title: String::from("Workflows"),
            },
            workflows: vec![Workflow {
                workflow_id: String::from("workflowFoo"),
                description: None,
                steps: vec![
                    Step {
                        step_id: String::from("stepFoo"),
                        description: None,
                        success_criteria: None,
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
                        description: None,
                        success_criteria: None,
                        on_success: None,
                        on_failure: None,
                    },
                    Step {
                        step_id: String::from("stepBaz"),
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
            "title: Workflows\n",
            "---\n",
            "flowchart TD\n",
            "    subgraph workflowFoo\n",
            "    stepFoo --> stepFooNode\n",
            "    stepFooNode -->|true| stepBar\n",
            "    stepFooNode -->|false| stepBaz\n",
            "    stepBar --> stepBaz\n",
            "    stepBaz --> workflowFooEndNode((End))\n",
            "    end\n",
        );

        assert_eq!(expected, actual);
    }

    #[test]
    fn render_success_criteria_omitted_on_success_defined_on_failure_omitted() {
        let arazzo = ArazzoDocument {
            info: Info {
                title: String::from("Workflows"),
            },
            workflows: vec![Workflow {
                workflow_id: String::from("workflowFoo"),
                description: None,
                steps: vec![
                    Step {
                        step_id: String::from("stepFoo"),
                        description: None,
                        success_criteria: None,
                        on_success: Some(vec![Action {
                            action_type: ActionType::Goto,
                            step_id: Some(String::from("stepBar")),
                        }]),
                        on_failure: None,
                    },
                    Step {
                        step_id: String::from("stepBar"),
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
            "title: Workflows\n",
            "---\n",
            "flowchart TD\n",
            "    subgraph workflowFoo\n",
            "    stepFoo --> stepFooNode\n",
            "    stepFooNode -->|true| stepBar\n",
            "    stepFooNode -->|false| workflowFooEndNode((End))\n",
            "    stepBar --> workflowFooEndNode((End))\n",
            "    end\n",
        );

        assert_eq!(expected, actual);
    }

    #[test]
    fn render_success_criteria_omitted_on_success_omitted_on_failure_defined() {
        let arazzo = ArazzoDocument {
            info: Info {
                title: String::from("Workflows"),
            },
            workflows: vec![Workflow {
                workflow_id: String::from("workflowFoo"),
                description: None,
                steps: vec![
                    Step {
                        step_id: String::from("stepFoo"),
                        description: None,
                        success_criteria: None,
                        on_success: None,
                        on_failure: Some(vec![Action {
                            action_type: ActionType::Goto,
                            step_id: Some(String::from("stepBaz")),
                        }]),
                    },
                    Step {
                        step_id: String::from("stepBar"),
                        description: None,
                        success_criteria: None,
                        on_success: None,
                        on_failure: None,
                    },
                    Step {
                        step_id: String::from("stepBaz"),
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
            "title: Workflows\n",
            "---\n",
            "flowchart TD\n",
            "    subgraph workflowFoo\n",
            "    stepFoo --> stepFooNode\n",
            "    stepFooNode -->|true| stepBar\n",
            "    stepFooNode -->|false| stepBaz\n",
            "    stepBar --> stepBaz\n",
            "    stepBaz --> workflowFooEndNode((End))\n",
            "    end\n",
        );

        assert_eq!(expected, actual);
    }

    #[test]
    fn render_success_criteria_omitted_on_success_omitted_on_failure_omitted() {
        let arazzo = ArazzoDocument {
            info: Info {
                title: String::from("Workflows"),
            },
            workflows: vec![Workflow {
                workflow_id: String::from("workflowFoo"),
                description: None,
                steps: vec![
                    Step {
                        step_id: String::from("stepFoo"),
                        description: None,
                        success_criteria: None,
                        on_success: None,
                        on_failure: None,
                    },
                    Step {
                        step_id: String::from("stepBar"),
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
            "title: Workflows\n",
            "---\n",
            "flowchart TD\n",
            "    subgraph workflowFoo\n",
            "    stepFoo --> stepBar\n",
            "    stepBar --> workflowFooEndNode((End))\n",
            "    end\n",
        );

        assert_eq!(expected, actual);
    }

    #[test]
    fn render_on_success_omitted_on_last_node() {
        let arazzo = ArazzoDocument {
            info: Info {
                title: String::from("Workflows"),
            },
            workflows: vec![Workflow {
                workflow_id: String::from("workflowFoo"),
                description: None,
                steps: vec![Step {
                    step_id: String::from("stepFoo"),
                    description: None,
                    success_criteria: Some(vec![Criteria {
                        condition: Some(String::from("$statusCode == 200")),
                    }]),
                    on_success: None,
                    on_failure: None,
                }],
            }],
        };

        let sut = MermaidFlowchart;

        let actual = sut.render(&arazzo);

        let expected = concat!(
            "---\n",
            "title: Workflows\n",
            "---\n",
            "flowchart TD\n",
            "    subgraph workflowFoo\n",
            "    stepFoo --> stepFooNode{$statusCode == 200}\n",
            "    stepFooNode{$statusCode == 200} -->|true| workflowFooEndNode((End))\n",
            "    stepFooNode{$statusCode == 200} -->|false| workflowFooEndNode((End))\n",
            "    end\n",
        );

        assert_eq!(expected, actual);
    }

    #[test]
    fn render_multiple_criteria_conditions() {
        let arazzo = ArazzoDocument {
            info: Info {
                title: String::from("Workflows"),
            },
            workflows: vec![Workflow {
                workflow_id: String::from("workflowFoo"),
                description: None,
                steps: vec![
                    Step {
                        step_id: String::from("stepFoo"),
                        description: None,
                        success_criteria: Some(vec![
                            Criteria {
                                condition: Some(String::from("$statusCode == 200")),
                            },
                            Criteria {
                                condition: Some(String::from("$response.body.status == done")),
                            },
                        ]),
                        on_success: None,
                        on_failure: None,
                    },
                    Step {
                        step_id: String::from("stepBar"),
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
            "title: Workflows\n",
            "---\n",
            "flowchart TD\n",
            "    subgraph workflowFoo\n",
            "    stepFoo --> stepFooNode{$statusCode == 200 && $response.body.status == done}\n",
            "    stepFooNode{$statusCode == 200 && $response.body.status == done} -->|true| stepBar\n",
            "    stepFooNode{$statusCode == 200 && $response.body.status == done} -->|false| workflowFooEndNode((End))\n",
            "    stepBar --> workflowFooEndNode((End))\n",
            "    end\n",
        );

        assert_eq!(expected, actual);
    }
}
