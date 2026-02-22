use crate::arazzo::{ArazzoDocument, Info, Step, Workflow};

pub trait Renderer {
    fn render(&self, document: &ArazzoDocument) -> String;
}

pub struct MermaidFlowchart;

impl Renderer for MermaidFlowchart {
    fn render(&self, arazzo: &ArazzoDocument) -> String {
        let mut output = title(&arazzo);
        output.push_str("flowchart TD\n");

        for workflow in &arazzo.workflows {
            output.push_str(&format!("    subgraph {}\n", workflow.workflow_id));

            for pair in workflow.steps.windows(2) {
                let from = &pair[0];
                let to = &pair[1];
                output.push_str(&format!(
                    "    {}{} --> {}{}\n",
                    from.step_id,
                    node_label(&from),
                    to.step_id,
                    node_label(&to),
                ));
            }

            output.push_str("    end\n");
        }

        output
    }
}

fn title(arazzo: &ArazzoDocument) -> String {
    format!("---\ntitle: {}\n---\n", arazzo.info.title)
}

fn node_label(step: &Step) -> String {
    step.description
        .as_ref()
        .map_or(String::from(""), |v| format!("[\"{}\"]", v))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_steps() {
        let arazzo = ArazzoDocument {
            info: Info {
                title: String::from("workflows"),
            },
            workflows: vec![Workflow {
                workflow_id: String::from("workflow_foo"),
                steps: vec![
                    Step {
                        step_id: String::from("step_foo"),
                        description: Some(String::from("description_foo")),
                    },
                    Step {
                        step_id: String::from("step_bar"),
                        description: Some(String::from("description_bar")),
                    },
                    Step {
                        step_id: String::from("step_baz"),
                        description: Some(String::from("description_baz")),
                    },
                ],
                ..Default::default()
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
            "    step_foo[\"description_foo\"] --> step_bar[\"description_bar\"]\n",
            "    step_bar[\"description_bar\"] --> step_baz[\"description_baz\"]\n",
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
                steps: vec![
                    Step {
                        step_id: String::from("step_foo"),
                        ..Default::default()
                    },
                    Step {
                        step_id: String::from("step_bar"),
                        ..Default::default()
                    },
                ],
                ..Default::default()
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
