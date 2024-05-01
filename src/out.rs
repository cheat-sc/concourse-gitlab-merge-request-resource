mod common;
use anyhow::{
	anyhow,
	Context,
	Result,
};
use clap::Parser;
use common::*;
use gitlab::{
	api::{
		projects::{
			merge_requests,
			repository::commits,
		},
		Query,
	},
	Gitlab,
};
use serde::{
	Deserialize,
	Serialize,
};
use std::env;
use std::fs::File;
use std::io;
use std::path::Path;
use url::Url;

#[derive(Debug, Deserialize)]
struct Params {
	resource_name: String,
	status: String,
	pipeline_name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ResourceInput {
	source: Source,
	params: Params,
}

#[derive(Debug, Serialize)]
struct ResourceOutput {
	version: Version,
	metadata: Vec<Metadata>,
}

#[derive(Parser)]
struct Args {
	#[arg()]
	directory: String,
}

fn compose_params_from_instance_vars(
	instance_vars: &serde_json::Map<String, serde_json::Value>,
	parent: Option<&String>,
) -> Option<String> {
	let mut params = Vec::<String>::new();

	/* NOTE: instance vars always dictionary */
	for (key, value) in instance_vars.iter() {
		let param = if let Some(parent) = parent {
			format!("{}.{}", parent, key)
		} else {
			key.to_owned()
		};

		if value.is_object() {
			params.push(compose_params_from_instance_vars(value.as_object().unwrap(), Some(&param)).unwrap());
		} else {
			params.push(format!("vars.{}={}", &param, &value).replace('"', "%22"));
		}
	}

	if params.is_empty() {
		None
	} else {
		Some(params.join("&"))
	}
}

fn main() -> Result<()> {
	let args = Args::parse();

	let input: ResourceInput =
		get_data_from(&mut io::stdin()).map_err(|err| anyhow!("{}", err.downcast::<serde_json::Error>().unwrap()))?;
	let version: Version = serde_json::from_reader(File::open(
		Path::new(&args.directory)
			.join(&input.params.resource_name)
			.join(".merge-request.json"),
	)?)
	.with_context(|| anyhow!("failed to read `.merge-request.json`"))?;

	let uri = Url::parse(&input.source.uri)?;
	let client = Gitlab::new(uri.host_str().unwrap(), &input.source.private_token)?;

	let mr: MergeRequest = merge_requests::MergeRequest::builder()
		.project(uri.path().trim_start_matches('/').trim_end_matches(".git"))
		.merge_request(version.iid.parse::<u64>().unwrap())
		.build()?
		.query(&client)?;

	/* get environment variables */
	let build_pipeline_name =
		env::var("BUILD_PIPELINE_NAME").with_context(|| anyhow!("BUILD_PIPELINE_NAME is not set"))?;
	let build_job_name = env::var("BUILD_JOB_NAME").with_context(|| anyhow!("BUILD_JOB_NAME is not set"))?;
	let build_team_name = env::var("BUILD_TEAM_NAME").with_context(|| anyhow!("BUILD_TEAM_NAME is not set"))?;
	let build_name = env::var("BUILD_NAME").with_context(|| anyhow!("BUILD_NAME is not set"))?;
	let build_pipeline_instance_vars = match env::var("BUILD_PIPELINE_INSTANCE_VARS") {
		Ok(v) => {
			let instance_vars: serde_json::Value = serde_json::from_str(&v).unwrap();
			format!(
				"?{}",
				compose_params_from_instance_vars(instance_vars.as_object().unwrap(), None).unwrap()
			)
		},
		Err(_) => "".to_owned(),
	};

	let concourse_uri = format!(
		"{}/teams/{}/pipelines/{}/jobs/{}/builds/{}{}",
		env::var("ATC_EXTERNAL_URL").with_context(|| anyhow!("ATC_EXTERNAL_URL is not set"))?,
		&build_team_name,
		&build_pipeline_name,
		&build_job_name,
		&build_name,
		&build_pipeline_instance_vars,
	);

	let pipeline_name = if let Some(pipeline_name) = &input.params.pipeline_name {
		pipeline_name
			.clone()
			.replace("%BUILD_PIPELINE_NAME%", &build_pipeline_name)
			.replace("%BUILD_JOB_NAME%", &build_job_name)
			.replace("%BUILD_TEAM_NAME%", &build_team_name)
			.replace("%BUILD_PIPELINE_INSTANCE_VARS%", &build_pipeline_instance_vars)
	} else {
		format!("{}::{}", build_team_name, build_pipeline_name)
	};

	let response: CommitStatusResponce = commits::CreateCommitStatus::builder()
		.project(mr.source_project_id)
		.commit(&version.sha)
		.state(match input.params.status.as_str() {
			"canceled" => commits::CommitStatusState::Canceled,
			"running" => commits::CommitStatusState::Running,
			"pending" => commits::CommitStatusState::Pending,
			"failed" => commits::CommitStatusState::Failed,
			"success" => commits::CommitStatusState::Success,
			_ => panic!("invalid status"),
		})
		.name(&pipeline_name)
		.target_url(&concourse_uri)
		.build()?
		.query(&client)?;

	#[allow(clippy::redundant_field_names)]
	let output = ResourceOutput {
		version: version,
		metadata: vec![
			Metadata {
				name: "url".to_owned(),
				value: mr.web_url,
			},
			Metadata {
				name: "author".to_owned(),
				value: mr.author.name,
			},
			Metadata {
				name: "title".to_owned(),
				value: mr.title,
			},
			Metadata {
				name: "status".to_owned(),
				value: response.status,
			},
		],
	};
	println!("{}", serde_json::to_string_pretty(&output)?);
	Ok(())
}

#[cfg(test)]
mod compose_params_from_instance_vars_tests {
	use super::*;

	#[test]
	fn test_generate_url_with_no_parameter() {
		let json = serde_json::json!({});
		let url = compose_params_from_instance_vars(json.as_object().unwrap(), None);
		assert!(url.is_none());
	}

	#[test]
	fn test_generate_url_with_a_string_parameter() {
		let json = serde_json::json!({ "a": "s" });
		let url = compose_params_from_instance_vars(json.as_object().unwrap(), None);
		assert!(url == Some("vars.a=%22s%22".to_owned()));
	}

	#[test]
	fn test_generate_url_with_a_integer_parameter() {
		let json = serde_json::json!({ "a": 0 });
		let url = compose_params_from_instance_vars(json.as_object().unwrap(), None);
		assert!(url == Some("vars.a=0".to_owned()));
	}

	#[test]
	fn test_generate_url_with_a_boolean_parameter() {
		let json = serde_json::json!({ "a": true });
		let url = compose_params_from_instance_vars(json.as_object().unwrap(), None);
		assert!(url == Some("vars.a=true".to_owned()));
	}

	#[test]
	fn test_generate_url_with_nested_parameter() {
		let json = serde_json::json!({
			"a": {
				"a": 0,
				"b": true
			}
		});
		let url = compose_params_from_instance_vars(json.as_object().unwrap(), None);
		assert!(url == Some("vars.a.a=0&vars.a.b=true".to_owned()));
	}

	#[test]
	fn test_generate_url_with_more_nested_parameter() {
		let json = serde_json::json!({
			"a": {
				"a": 0,
				"b": {
					"c": 0,
					"d": {
						"e": 0
					}
				}
			}
		});
		let url = compose_params_from_instance_vars(json.as_object().unwrap(), None);
		assert!(url == Some("vars.a.a=0&vars.a.b.c=0&vars.a.b.d.e=0".to_owned()));
	}

	#[test]
	fn test_generate_url_with_complex_parameter() {
		let json = serde_json::json!({
			"a": 0,
			"b": {
				"a": 0,
				"b": true
			},
			"c": "0-0"
		});
		let url = compose_params_from_instance_vars(json.as_object().unwrap(), None);
		assert!(url == Some("vars.a=0&vars.b.a=0&vars.b.b=true&vars.c=%220-0%22".to_owned()));
	}
}
