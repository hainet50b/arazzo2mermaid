use std::error::Error;
use std::{env, fs, io};
use std::io::Read;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ArazzoDocument {
    workflows: Vec<Workflow>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Workflow {
    workflow_id: String,
    summary: Option<String>,
    steps: Vec<Step>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Step {
    step_id: String,
}

trait Renderer {
    fn render(&self, document: &ArazzoDocument) -> String;
}

struct MermaidFlowchart;

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

fn main() -> Result<(), Box<dyn Error>> {
    let args = env::args().skip(1);
    run(args)?;

    Ok(())
}

fn run(mut args: impl Iterator<Item = String>) -> Result<(), Box<dyn Error>> {
    let path = args.next();

    if args.next().is_some() {
        return Err("Too many arguments".into());
    }

    let mut reader: Box<dyn Read> = match path.as_deref() {
        None | Some("-") => Box::new(io::stdin()),
        Some(path) => Box::new(fs::File::open(path)?),
    };

    let mut content = String::new();
    reader.read_to_string(&mut content)?;

    let arazzo: ArazzoDocument = yaml_serde::from_str(&content)?;
    let renderer = MermaidFlowchart;
    print!("{}", renderer.render(&arazzo));

    Ok(())
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

