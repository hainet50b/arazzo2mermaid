use crate::arazzo::{ArazzoDocument, Step, Workflow};

pub trait Renderer {
    fn render(&self, document: &ArazzoDocument) -> String;
}

pub struct MermaidFlowchart;

impl Renderer for MermaidFlowchart {
    fn render(&self, document: &ArazzoDocument) -> String {
        let mut output = String::new();

        for workflow in &document.workflows {
            output.push_str(&title(&workflow));

            output.push_str("flowchart TD\n");

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
        }

        output
    }
}

fn title(workflow: &Workflow) -> String {
    format!("---\ntitle: {}\n---\n", workflow.workflow_id)
}

fn node_label(step: &Step) -> String {
    step.description.as_ref().map_or(String::from(""), |v| format!("[\"{}\"]", v))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_steps() {
        let arazzo = ArazzoDocument {
            workflows: vec![Workflow {
                workflow_id: String::from("workflow_a"),
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
            "title: workflow_a\n",
            "---\n",
            "flowchart TD\n",
            "    step_foo[\"description_foo\"] --> step_bar[\"description_bar\"]\n",
            "    step_bar[\"description_bar\"] --> step_baz[\"description_baz\"]\n",
        );

        assert_eq!(expected, actual);
    }

    #[test]
    fn render_steps_without_description() {
        let arazzo = ArazzoDocument {
            workflows: vec![Workflow {
                workflow_id: String::from("workflow_a"),
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
            "title: workflow_a\n",
            "---\n",
            "flowchart TD\n",
            "    step_foo --> step_bar\n",
        );

        assert_eq!(expected, actual);
    }
}
