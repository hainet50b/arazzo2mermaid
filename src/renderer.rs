use crate::arazzo::{Action, ActionType, ArazzoDocument, Criteria, Step};

pub trait Renderer {
    fn render(&self, document: &ArazzoDocument) -> String;
}

pub struct MermaidFlowchart;

impl Renderer for MermaidFlowchart {
    fn render(&self, arazzo: &ArazzoDocument) -> String {
        let mut output = title(arazzo.info.title.as_ref());
        output.push_str("flowchart TD\n");

        for workflow in &arazzo.workflows {
            output.push_str(&subgraph(
                &workflow.workflow_id,
                workflow.description.as_deref(),
            ));

            for (i, current_step) in workflow.steps.iter().enumerate() {
                if should_branch(current_step) {
                    output.push_str(&to_rhombus_from_rectangle(
                        &rectangle_node(&current_step.step_id, &current_step.description),
                        &rhombus_node(&current_step.step_id, &current_step.success_criteria),
                    ));

                    if let Some(on_success) =
                        current_step.on_success.as_deref().and_then(|a| a.first())
                    {
                        output.push_str(&render_action(
                            &current_step.step_id,
                            &current_step.success_criteria,
                            on_success,
                            ActionSide::OnSuccess,
                            &workflow.workflow_id,
                        ));
                    } else if let Some(next_step) = &workflow.steps.get(i + 1) {
                        output.push_str(&to_rectangle_from_rhombus(
                            &rhombus_node(&current_step.step_id, &current_step.success_criteria),
                            &rectangle_node(&next_step.step_id, &next_step.description),
                            Verdict::Ok,
                        ));
                    } else {
                        output.push_str(&to_end_from_rhombus(
                            &rhombus_node(&current_step.step_id, &current_step.success_criteria),
                            &end_node(&workflow.workflow_id),
                            Verdict::Ok,
                        ));
                    }

                    if let Some(on_failure) =
                        current_step.on_failure.as_deref().and_then(|a| a.first())
                    {
                        output.push_str(&render_action(
                            &current_step.step_id,
                            &current_step.success_criteria,
                            on_failure,
                            ActionSide::OnFailure,
                            &workflow.workflow_id,
                        ));
                    } else {
                        output.push_str(&to_end_from_rhombus(
                            &rhombus_node(&current_step.step_id, &current_step.success_criteria),
                            &end_node(&workflow.workflow_id),
                            Verdict::Ng,
                        ));
                    }
                } else if let Some(next_step) = &workflow.steps.get(i + 1) {
                    output.push_str(&to_rectangle_from_rectangle(
                        &rectangle_node(&current_step.step_id, &current_step.description),
                        &rectangle_node(&next_step.step_id, &next_step.description),
                    ));
                } else {
                    output.push_str(&to_end_from_rectangle(
                        &rectangle_node(&current_step.step_id, &current_step.description),
                        &end_node(&workflow.workflow_id),
                    ));
                }
            }

            output.push_str("    end\n");
        }

        output
    }
}

fn render_action(
    step_id: &str,
    success_criteria: &Option<Vec<Criteria>>,
    action: &Action,
    action_side: ActionSide,
    workflow_id: &str,
) -> String {
    let mut output = String::new();

    let mut from_rhombus_node = rhombus_node(step_id, success_criteria);
    let mut verdict = match action_side {
        ActionSide::OnSuccess => Verdict::Ok,
        ActionSide::OnFailure => Verdict::Ng,
    };
    let has_criteria = action.criteria.is_some();

    if has_criteria {
        let to_rhombus_node = rhombus_node(&action.name, &action.criteria);

        output.push_str(&to_rhombus_from_rhombus(
            &from_rhombus_node,
            &to_rhombus_node,
            verdict,
        ));

        from_rhombus_node = to_rhombus_node;
        verdict = Verdict::Ok
    }

    match action.action_type {
        ActionType::Goto => {
            if let Some(action_target_id) =
                action.workflow_id.as_deref().or(action.step_id.as_deref())
            {
                output.push_str(&to_rectangle_from_rhombus(
                    &from_rhombus_node,
                    &rectangle_node(action_target_id, &None),
                    verdict,
                ));
            }
        }
        ActionType::End => {
            output.push_str(&to_end_from_rhombus(
                &from_rhombus_node,
                &end_node(workflow_id),
                verdict,
            ));
        }
    }

    if has_criteria {
        output.push_str(&to_end_from_rhombus(
            &from_rhombus_node,
            &end_node(workflow_id),
            Verdict::Ng,
        ));
    }

    output
}

fn should_branch(step: &Step) -> bool {
    step.success_criteria.is_some() || step.on_success.is_some() || step.on_failure.is_some()
}

fn title(graph_title: &str) -> String {
    format!("---\ntitle: {graph_title}\n---\n")
}

fn subgraph(title: &str, description: Option<&str>) -> String {
    format!(
        "    subgraph {subgraph_title}{subgraph_description}\n",
        subgraph_title = title,
        subgraph_description = subgraph_description(description)
    )
}

fn subgraph_description(subgraph_description: Option<&str>) -> String {
    subgraph_description.map_or(String::new(), |v| format!("[\"{}\"]", v))
}

fn to_rectangle_from_rectangle(from: &str, to: &str) -> String {
    format!(
        "    {from_node} --> {to_node}\n",
        from_node = from,
        to_node = to,
    )
}

fn to_rectangle_from_rhombus(from: &str, to: &str, verdict: Verdict) -> String {
    format!(
        "    {rhombus_node} -->|{verdict}| {rectangle_node}\n",
        rhombus_node = from,
        rectangle_node = to,
        verdict = match verdict {
            Verdict::Ok => "true",
            Verdict::Ng => "false",
        },
    )
}

fn to_rhombus_from_rectangle(from: &str, to: &str) -> String {
    format!(
        "    {rectangle_node} --> {rhombus_node}\n",
        rectangle_node = from,
        rhombus_node = to,
    )
}

fn to_rhombus_from_rhombus(from: &str, to: &str, verdict: Verdict) -> String {
    format!(
        "    {from_node} -->|{verdict}| {to_node}\n",
        from_node = from,
        to_node = to,
        verdict = match verdict {
            Verdict::Ok => "true",
            Verdict::Ng => "false",
        },
    )
}

fn to_end_from_rectangle(from: &str, to: &str) -> String {
    format!(
        "    {rectangle_node} --> {end_node}\n",
        rectangle_node = from,
        end_node = to,
    )
}

fn to_end_from_rhombus(from: &str, to: &str, verdict: Verdict) -> String {
    format!(
        "    {rhombus_node} -->|{verdict}| {end_node}\n",
        rhombus_node = from,
        end_node = to,
        verdict = match verdict {
            Verdict::Ok => "true",
            Verdict::Ng => "false",
        },
    )
}

#[derive(Clone, Copy)]
enum Verdict {
    Ok,
    Ng,
}

#[derive(Clone, Copy)]
enum ActionSide {
    OnSuccess,
    OnFailure,
}

fn rectangle_node(node_name: &str, node_label: &Option<String>) -> String {
    format!(
        "{node_name}{node_label}",
        node_label = rectangle_node_label(node_label),
    )
}

fn rectangle_node_label(node_label: &Option<String>) -> String {
    node_label
        .as_deref()
        .map_or(String::new(), |v| format!("[\"{}\"]", v))
}

fn rhombus_node(node_name: &str, criteria: &Option<Vec<Criteria>>) -> String {
    format!(
        "{node_name}Node{condition}",
        condition = rhombus_node_condition(criteria),
    )
}

fn rhombus_node_condition(criteria: &Option<Vec<Criteria>>) -> String {
    if let Some(criteria) = &criteria
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

fn end_node(node_name: &str) -> String {
    format!("{node_name}EndNode((End))")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arazzo::{Criteria, Info, Workflow};

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
                                name: String::from("proceedToStepBar"),
                                action_type: ActionType::Goto,
                                workflow_id: None,
                                step_id: Some(String::from("stepBar")),
                                criteria: None,
                            }]),
                            on_failure: Some(vec![Action {
                                name: String::from("proceedToStepBaz"),
                                action_type: ActionType::Goto,
                                workflow_id: None,
                                step_id: Some(String::from("stepBaz")),
                                criteria: None,
                            }]),
                        },
                        Step {
                            step_id: String::from("stepBar"),
                            description: Some(String::from("Step bar's description.")),
                            success_criteria: Some(vec![Criteria {
                                condition: Some(String::from("$statusCode == 200")),
                            }]),
                            on_success: Some(vec![Action {
                                name: String::from("done"),
                                action_type: ActionType::End,
                                workflow_id: None,
                                step_id: None,
                                criteria: None,
                            }]),
                            on_failure: Some(vec![Action {
                                name: String::from("proceedToStepBaz"),
                                action_type: ActionType::Goto,
                                workflow_id: None,
                                step_id: Some(String::from("stepBaz")),
                                criteria: None,
                            }]),
                        },
                        Step {
                            step_id: String::from("stepBaz"),
                            description: Some(String::from("Step baz's description.")),
                            success_criteria: Some(vec![Criteria {
                                condition: Some(String::from("$statusCode == 200")),
                            }]),
                            on_success: Some(vec![Action {
                                name: String::from("proceedToWorkflowBar"),
                                action_type: ActionType::Goto,
                                workflow_id: Some(String::from("workflowBar")),
                                step_id: None,
                                criteria: None,
                            }]),
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
                                name: String::from("proceedToStepBar"),
                                action_type: ActionType::Goto,
                                workflow_id: None,
                                step_id: Some(String::from("stepBar")),
                                criteria: Some(vec![Criteria {
                                    condition: Some(String::from(
                                        "$response.body.status == 'approved'",
                                    )),
                                }]),
                            }]),
                            on_failure: Some(vec![Action {
                                name: String::from("proceedToStepBaz"),
                                action_type: ActionType::Goto,
                                workflow_id: None,
                                step_id: Some(String::from("stepBaz")),
                                criteria: Some(vec![Criteria {
                                    condition: Some(String::from("$response.body.error != null")),
                                }]),
                            }]),
                        },
                        Step {
                            step_id: String::from("stepBar"),
                            description: Some(String::from("Step bar's description.")),
                            success_criteria: Some(vec![Criteria {
                                condition: Some(String::from("$statusCode == 200")),
                            }]),
                            on_success: Some(vec![Action {
                                name: String::from("done"),
                                action_type: ActionType::End,
                                workflow_id: None,
                                step_id: None,
                                criteria: None,
                            }]),
                            on_failure: Some(vec![Action {
                                name: String::from("proceedToStepBaz"),
                                action_type: ActionType::Goto,
                                workflow_id: None,
                                step_id: Some(String::from("stepBaz")),
                                criteria: None,
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
            "    stepBaz[\"Step baz's description.\"] --> stepBazNode{$statusCode == 200}\n",
            "    stepBazNode{$statusCode == 200} -->|true| workflowBar\n",
            "    stepBazNode{$statusCode == 200} -->|false| workflowFooEndNode((End))\n",
            "    end\n",
            "    subgraph workflowBar[\"Workflow bar's description.\"]\n",
            "    stepFoo[\"Step foo's description.\"] --> stepFooNode{$statusCode == 200}\n",
            "    stepFooNode{$statusCode == 200} -->|true| proceedToStepBarNode{$response.body.status == 'approved'}\n",
            "    proceedToStepBarNode{$response.body.status == 'approved'} -->|true| stepBar\n",
            "    proceedToStepBarNode{$response.body.status == 'approved'} -->|false| workflowBarEndNode((End))\n",
            "    stepFooNode{$statusCode == 200} -->|false| proceedToStepBazNode{$response.body.error != null}\n",
            "    proceedToStepBazNode{$response.body.error != null} -->|true| stepBaz\n",
            "    proceedToStepBazNode{$response.body.error != null} -->|false| workflowBarEndNode((End))\n",
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
                            name: String::from("proceedToStepBar"),
                            action_type: ActionType::Goto,
                            workflow_id: None,
                            step_id: Some(String::from("stepBar")),
                            criteria: None,
                        }]),
                        on_failure: Some(vec![Action {
                            name: String::from("proceedToStepBaz"),
                            action_type: ActionType::Goto,
                            workflow_id: None,
                            step_id: Some(String::from("stepBaz")),
                            criteria: None,
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
                            name: String::from("proceedToStepBar"),
                            action_type: ActionType::Goto,
                            workflow_id: None,
                            step_id: Some(String::from("stepBar")),
                            criteria: None,
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
                            name: String::from("proceedToStepBaz"),
                            action_type: ActionType::Goto,
                            workflow_id: None,
                            step_id: Some(String::from("stepBaz")),
                            criteria: None,
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
                            name: String::from("proceedToStepBar"),
                            action_type: ActionType::Goto,
                            workflow_id: None,
                            step_id: Some(String::from("stepBar")),
                            criteria: None,
                        }]),
                        on_failure: Some(vec![Action {
                            name: String::from("proceedToStepBaz"),
                            action_type: ActionType::Goto,
                            workflow_id: None,
                            step_id: Some(String::from("stepBaz")),
                            criteria: None,
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
                            name: String::from("proceedToStepBar"),
                            action_type: ActionType::Goto,
                            workflow_id: None,
                            step_id: Some(String::from("stepBar")),
                            criteria: None,
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
                            name: String::from("proceedToStepBaz"),
                            action_type: ActionType::Goto,
                            workflow_id: None,
                            step_id: Some(String::from("stepBaz")),
                            criteria: None,
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
    fn render_success_criteria_defined_with_multiple_criteria() {
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

    #[test]
    fn render_success_criteria_defined_with_empty_criteria() {
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
                        success_criteria: Some(vec![]),
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
            "    stepFoo --> stepFooNode\n",
            "    stepFooNode -->|true| stepBar\n",
            "    stepFooNode -->|false| workflowFooEndNode((End))\n",
            "    stepBar --> workflowFooEndNode((End))\n",
            "    end\n",
        );

        assert_eq!(expected, actual);
    }

    #[test]
    fn render_success_criteria_defined_on_success_defined_as_action_type_end() {
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
                    on_success: Some(vec![Action {
                        name: String::from("done"),
                        action_type: ActionType::End,
                        workflow_id: None,
                        step_id: None,
                        criteria: None,
                    }]),
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
    fn render_success_criteria_defined_on_success_omitted_on_last_node() {
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
    fn render_on_action() {
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
                            name: String::from("proceedToStepBar"),
                            action_type: ActionType::Goto,
                            workflow_id: None,
                            step_id: Some(String::from("stepBar")),
                            criteria: Some(vec![Criteria {
                                condition: Some(String::from(
                                    "$response.body.status == 'approved'",
                                )),
                            }]),
                        }]),
                        on_failure: Some(vec![Action {
                            name: String::from("done"),
                            action_type: ActionType::End,
                            workflow_id: None,
                            step_id: None,
                            criteria: Some(vec![Criteria {
                                condition: Some(String::from("$response.body.error != null")),
                            }]),
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
            "    stepFooNode{$statusCode == 200} -->|true| proceedToStepBarNode{$response.body.status == 'approved'}\n",
            "    proceedToStepBarNode{$response.body.status == 'approved'} -->|true| stepBar\n",
            "    proceedToStepBarNode{$response.body.status == 'approved'} -->|false| workflowFooEndNode((End))\n",
            "    stepFooNode{$statusCode == 200} -->|false| doneNode{$response.body.error != null}\n",
            "    doneNode{$response.body.error != null} -->|true| workflowFooEndNode((End))\n",
            "    doneNode{$response.body.error != null} -->|false| workflowFooEndNode((End))\n",
            "    stepBar --> stepBaz\n",
            "    stepBaz --> workflowFooEndNode((End))\n",
            "    end\n",
        );

        assert_eq!(expected, actual);
    }

    #[test]
    fn render_on_action_goto() {
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
                            name: String::from("proceedToStepBar"),
                            action_type: ActionType::Goto,
                            workflow_id: None,
                            step_id: Some(String::from("stepBar")),
                            criteria: Some(vec![Criteria {
                                condition: Some(String::from(
                                    "$response.body.status == 'approved'",
                                )),
                            }]),
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
            "    stepFooNode{$statusCode == 200} -->|true| proceedToStepBarNode{$response.body.status == 'approved'}\n",
            "    proceedToStepBarNode{$response.body.status == 'approved'} -->|true| stepBar\n",
            "    proceedToStepBarNode{$response.body.status == 'approved'} -->|false| workflowFooEndNode((End))\n",
            "    stepFooNode{$statusCode == 200} -->|false| workflowFooEndNode((End))\n",
            "    stepBar --> workflowFooEndNode((End))\n",
            "    end\n",
        );

        assert_eq!(expected, actual);
    }

    #[test]
    fn render_on_action_end() {
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
                    on_success: Some(vec![Action {
                        name: String::from("done"),
                        action_type: ActionType::End,
                        workflow_id: None,
                        step_id: None,
                        criteria: Some(vec![Criteria {
                            condition: Some(String::from("$response.body.status == 'approved'")),
                        }]),
                    }]),
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
            "    stepFooNode{$statusCode == 200} -->|true| doneNode{$response.body.status == 'approved'}\n",
            "    doneNode{$response.body.status == 'approved'} -->|true| workflowFooEndNode((End))\n",
            "    doneNode{$response.body.status == 'approved'} -->|false| workflowFooEndNode((End))\n",
            "    stepFooNode{$statusCode == 200} -->|false| workflowFooEndNode((End))\n",
            "    end\n",
        );

        assert_eq!(expected, actual);
    }

    #[test]
    fn render_on_action_on_success_only() {
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
                            name: String::from("proceedToStepBar"),
                            action_type: ActionType::Goto,
                            workflow_id: None,
                            step_id: Some(String::from("stepBar")),
                            criteria: Some(vec![Criteria {
                                condition: Some(String::from(
                                    "$response.body.status == 'approved'",
                                )),
                            }]),
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
            "    stepFooNode{$statusCode == 200} -->|true| proceedToStepBarNode{$response.body.status == 'approved'}\n",
            "    proceedToStepBarNode{$response.body.status == 'approved'} -->|true| stepBar\n",
            "    proceedToStepBarNode{$response.body.status == 'approved'} -->|false| workflowFooEndNode((End))\n",
            "    stepFooNode{$statusCode == 200} -->|false| workflowFooEndNode((End))\n",
            "    stepBar --> workflowFooEndNode((End))\n",
            "    end\n",
        );

        assert_eq!(expected, actual);
    }

    #[test]
    fn render_on_action_on_failure_only() {
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
                            name: String::from("proceedToStepBar"),
                            action_type: ActionType::Goto,
                            workflow_id: None,
                            step_id: Some(String::from("stepBar")),
                            criteria: Some(vec![Criteria {
                                condition: Some(String::from("$response.body.error != null")),
                            }]),
                        }]),
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
            "    stepFooNode{$statusCode == 200} -->|false| proceedToStepBarNode{$response.body.error != null}\n",
            "    proceedToStepBarNode{$response.body.error != null} -->|true| stepBar\n",
            "    proceedToStepBarNode{$response.body.error != null} -->|false| workflowFooEndNode((End))\n",
            "    stepBar --> workflowFooEndNode((End))\n",
            "    end\n",
        );

        assert_eq!(expected, actual);
    }

    #[test]
    fn render_on_action_with_empty_criteria() {
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
                            name: String::from("proceedToStepBar"),
                            action_type: ActionType::Goto,
                            workflow_id: None,
                            step_id: Some(String::from("stepBar")),
                            criteria: Some(vec![]),
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
            "    stepFooNode{$statusCode == 200} -->|true| proceedToStepBarNode\n",
            "    proceedToStepBarNode -->|true| stepBar\n",
            "    proceedToStepBarNode -->|false| workflowFooEndNode((End))\n",
            "    stepFooNode{$statusCode == 200} -->|false| workflowFooEndNode((End))\n",
            "    stepBar --> workflowFooEndNode((End))\n",
            "    end\n",
        );

        assert_eq!(expected, actual);
    }

    #[test]
    fn render_on_action_with_multiple_action_criteria() {
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
                            name: String::from("proceedToStepBar"),
                            action_type: ActionType::Goto,
                            workflow_id: None,
                            step_id: Some(String::from("stepBar")),
                            criteria: Some(vec![
                                Criteria {
                                    condition: Some(String::from(
                                        "$response.body.status == 'approved'",
                                    )),
                                },
                                Criteria {
                                    condition: Some(String::from("$response.body.error == null")),
                                },
                            ]),
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
            "    stepFooNode -->|true| proceedToStepBarNode{$response.body.status == 'approved' && $response.body.error == null}\n",
            "    proceedToStepBarNode{$response.body.status == 'approved' && $response.body.error == null} -->|true| stepBar\n",
            "    proceedToStepBarNode{$response.body.status == 'approved' && $response.body.error == null} -->|false| workflowFooEndNode((End))\n",
            "    stepFooNode -->|false| workflowFooEndNode((End))\n",
            "    stepBar --> workflowFooEndNode((End))\n",
            "    end\n",
        );

        assert_eq!(expected, actual);
    }

    #[test]
    fn render_on_action_goto_another_workflow() {
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
                        on_success: Some(vec![Action {
                            name: String::from("proceedToStepBar"),
                            action_type: ActionType::Goto,
                            workflow_id: Some(String::from("workflowBar")),
                            step_id: None,
                            criteria: None,
                        }]),
                        on_failure: Some(vec![Action {
                            name: String::from("proceedToStepBaz"),
                            action_type: ActionType::Goto,
                            workflow_id: Some(String::from("workflowBaz")),
                            step_id: None,
                            criteria: None,
                        }]),
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
                Workflow {
                    workflow_id: String::from("workflowBaz"),
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
            "    stepFoo --> stepFooNode\n",
            "    stepFooNode -->|true| workflowBar\n",
            "    stepFooNode -->|false| workflowBaz\n",
            "    end\n",
            "    subgraph workflowBar\n",
            "    stepFoo --> workflowBarEndNode((End))\n",
            "    end\n",
            "    subgraph workflowBaz\n",
            "    stepFoo --> workflowBazEndNode((End))\n",
            "    end\n",
        );

        assert_eq!(expected, actual);
    }
}
