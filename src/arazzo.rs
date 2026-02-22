use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArazzoDocument {
    pub info: Info,
    pub workflows: Vec<Workflow>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Info {
    pub title: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Workflow {
    pub workflow_id: String,
    pub description: Option<String>,
    pub steps: Vec<Step>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Step {
    pub step_id: String,
    pub description: Option<String>,
    pub success_criteria: Option<Vec<Criteria>>,
    pub on_success: Option<Vec<Action>>,
    pub on_failure: Option<Vec<Action>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Criteria {
    pub condition: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Action {
    #[serde(rename = "type")]
    pub action_type: ActionType,
    pub step_id: Option<String>,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ActionType {
    Goto,
    End,
}
