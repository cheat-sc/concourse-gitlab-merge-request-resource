use serde::{ Deserialize, Serialize };
use serde_json;
use std::io;
use std::error;

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct Params {
	pub status: Option<String>,
	pub coverage: Option<f32>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Metadata {
	pub name: String,
	pub value: String,
}

#[derive(Debug, Deserialize)]
pub struct CommitStatusResponce {
	pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct Commit {
	pub committed_date: String,
}

#[derive(Debug, Deserialize)]
pub struct Project {
	pub http_url_to_repo: String,
	pub ssh_url_to_repo: String,
}

#[derive(Debug, Deserialize)]
pub struct Author {
	pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct Change {
	pub new_path: String,
}

#[derive(Debug, Deserialize)]
pub struct MergeRequestChanges {
	pub changes: Vec<Change>,
}

#[derive(Debug, Deserialize)]
pub struct MergeRequest {
	pub iid: u64,
	pub title: String,
	pub labels: Vec<String>,
	pub sha: String,
	pub author: Author,
	pub updated_at: String,
	pub source_project_id: u64,
	pub source_branch: String,
	pub web_url: String,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct Version {
	pub iid: String,
	pub committed_date: String,
	pub sha: String,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct Source {
	pub uri: String,
	pub private_token: String,
	pub labels: Option<Vec<String>>,
	pub paths: Option<Vec<String>>,
	pub skip_draft: Option<bool>,
}

pub fn get_data_from<T: for<'de> Deserialize<'de>>(stdin: &mut impl io::Read) -> Result<T, Box<dyn error::Error>> {
	let mut buffer = String::new();
	stdin.read_to_string(&mut buffer)?;
	Ok(serde_json::from_str(&buffer)?)
}

#[cfg(test)]
mod tests {
    use super::{ get_data_from, Source, Deserialize, Version };

	#[test]
	fn test_get_data_from() {
		#[derive(Debug, Deserialize, PartialEq)]
		struct ResourceInput {
			source: Source,
			version: Option<Version>
		}

		let dummy = r#"
			{
				"source": {
					"uri": "https://gitlab.com/cheatsc/test.git",
					"private_token": "zzzzz"
				}
			}
		"#;
		assert_eq!(
			get_data_from::<ResourceInput>(&mut dummy.as_bytes()).unwrap(),
			ResourceInput {
				source: Source {
					uri: "https://gitlab.com/cheatsc/test.git".to_owned(),
					private_token: "zzzzz".to_owned(),
					labels: None,
					paths: None,
					skip_draft: None,
				},
				version: None,
			}
		);
	}
}
