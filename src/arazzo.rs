use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArazzoDocument {
    pub workflows: Vec<Workflow>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Workflow {
    pub workflow_id: String,
    pub summary: Option<String>,
    pub steps: Vec<Step>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Step {
    pub step_id: String,
    pub description: Option<String>,
}
