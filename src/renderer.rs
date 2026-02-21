use crate::arazzo::{ArazzoDocument, Workflow, Step};

pub trait Renderer {
    fn render(&self, document: &ArazzoDocument) -> String;
}

pub struct MermaidFlowchart;

impl Renderer for MermaidFlowchart {
    fn render(&self, document: &ArazzoDocument) -> String {
        let mut output = String::from("flowchart TD\n");

        for workflow in &document.workflows {
            for pair in workflow.steps.windows(2) {
                let from = &pair[0].step_id;
                let to = &pair[1].step_id;
                output.push_str(&format!("    {from} --> {to}\n"));
            }
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_steps_in_flowchart_manner() {
        let arazzo = ArazzoDocument {
            workflows: vec![
                Workflow {
                    workflow_id: String::from("workflow"),
                    summary: None,
                    steps: vec![
                        Step { step_id: String::from("step_foo") },
                        Step { step_id: String::from("step_bar") },
                        Step { step_id: String::from("step_baz") },
                    ]
                },
            ]
        };

        let expected = concat!(
            "flowchart TD\n",
            "    step_foo --> step_bar\n",
            "    step_bar --> step_baz\n",
        );

        let sut = MermaidFlowchart;

        let actual = sut.render(&arazzo);

        assert_eq!(expected, actual);
    }
}

