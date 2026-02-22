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

                    [Verdict::Ok, Verdict::Ng].iter()
                        .filter_map(|&v| lookup_actions(current_step, v).map(|a| (v, a)))
                        .flat_map(|(v, actions)| actions.iter().map(move |a| (v, a)))
                        .for_each(|(v, a)| {
                            match a.action_type {
                                ActionType::Goto => {
                                    if let Some(step_id) = &a.step_id {
                                        output.push_str(&to_rectangle_from_rhombus(
                                            current_step,
                                            v,
                                            step_id,
                                        ));
                                    }
                                },
                                ActionType::End => {
                                    output.push_str(&to_end_from_rhombus(current_step, v, workflow));
                                }
                            }
                        })
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

fn has_goto_actions(step: &Step) -> bool {
    [Verdict::Ok, Verdict::Ng].iter()
        .filter_map(|&v| lookup_actions(step, v))
        .flat_map(|actions| actions.iter())
        .any(|a| a.action_type == ActionType::Goto)
}

fn lookup_actions(step: &Step, verdict: Verdict) -> Option<&Vec<Action>> {
    match verdict {
        Verdict::Ok => step.on_success.as_ref(),
        Verdict::Ng => step.on_failure.as_ref(),
    }
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
        .map_or(String::from(""), |v| format!("[\"{}\"]", v))
}

fn rhombus_node(step: &Step) -> String {
    format!(
        "{node_name}Node{condition}",
        node_name = step.step_id,
        condition = condition(step),
    )
}

fn condition(step: &Step) -> String {
    step.success_criteria
        .as_ref()
        .and_then(|cs| cs.first())
        .and_then(|c| c.condition.as_ref())
        .map_or(String::from(""), |v| format!("{{{}}}", v))
}

fn end_node(workflow: &Workflow) -> String {
    format!("{}EndNode((End))", workflow.workflow_id)
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
                        step_id: String::from("stepBaz"),
                        description: Some(String::from("Step baz's description.")),
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
            "    stepBarNode{$statusCode == 200} -->|true| workflowFooEndNode((End))\n",
            "    stepBarNode{$statusCode == 200} -->|false| stepBaz\n",
            "    stepBaz[\"Step baz's description.\"] --> workflowFooEndNode((End))\n",
            "    end\n",
        );

        assert_eq!(expected, actual);
    }

    #[test]
    fn render_steps_without_description() {
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
}
