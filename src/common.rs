use serde::{
	Deserialize,
	Serialize,
};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[allow(dead_code)]
pub struct Params {
	pub status: Option<String>,
	pub coverage: Option<f32>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct CommitStatusResponce {
	pub status: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Commit {
	pub committed_date: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Project {
	pub http_url_to_repo: String,
	pub ssh_url_to_repo: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Author {
	pub name: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Change {
	pub new_path: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct MergeRequestChanges {
	pub changes: Vec<Change>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct MergeRequest {
	pub iid: u64,
	pub title: String,
	pub labels: Vec<String>,
	pub sha: String,
	pub author: Author,
	pub updated_at: String,
	pub source_project_id: u64,
	pub source_branch: String,
	pub target_branch: String,
	pub web_url: String,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct Version {
	pub iid: String,
	pub committed_date: String,
	pub sha: String,
	pub source_branch: String,
	pub target_branch: String,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct Source {
	pub uri: String,
	pub private_token: String,
	pub labels: Option<Vec<String>>,
	pub paths: Option<Vec<String>>,
	pub skip_draft: Option<bool>,
}
