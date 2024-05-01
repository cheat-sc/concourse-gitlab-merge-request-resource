mod common;
use anyhow::{
	anyhow,
	Context,
	Result,
};
use clap::Parser;
use common::*;
use git2::build::RepoBuilder;
use git2::{
	Cred,
	FetchOptions,
	Oid,
	RemoteCallbacks,
};
use gitlab::api::{
	projects::{
		self,
		merge_requests,
	},
	Query,
};
use gitlab::Gitlab;
use serde::{
	Deserialize,
	Serialize,
};
use serde_json;
use std::path::Path;
use std::{
	fs::File,
	io,
};
use url::Url;

#[derive(Debug, Deserialize)]
struct Params {
	skip_clone: Option<bool>,
}

impl Params {
	fn is_clone_skippable(&self) -> bool {
		self.skip_clone.is_some_and(|skip_clone| skip_clone)
	}
}

#[derive(Debug, Deserialize)]
struct ResourceInput {
	version: Option<Version>,
	source: Source,
	params: Option<Params>,
}

impl ResourceInput {
	pub fn is_clone_skippable(&self) -> bool {
		self.params.as_ref().is_some_and(|params| params.is_clone_skippable())
	}
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

fn main() -> Result<()> {
	let args = Args::parse();

	let input: ResourceInput =
		get_data_from(&mut io::stdin()).map_err(|err| anyhow!("{}", err.downcast::<serde_json::Error>().unwrap()))?;

	let uri = Url::parse(&input.source.uri)?;
	let client = Gitlab::new(uri.host_str().unwrap(), &input.source.private_token)?;

	let version = input.version.as_ref().unwrap();

	let mr: MergeRequest = merge_requests::MergeRequest::builder()
		.project(uri.path().trim_start_matches("/").trim_end_matches(".git"))
		.merge_request(version.iid.parse::<u64>()?)
		.build()?
		.query(&client)?;

	let project: Project = projects::Project::builder()
		.project(mr.source_project_id)
		.build()?
		.query(&client)?;

	let output = ResourceOutput {
		version: version.clone(),
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
		],
	};

	println!("{}", serde_json::to_string_pretty(&output)?);

	if !input.is_clone_skippable() {
		eprintln!("Cloning repository...");
		let mut cb = RemoteCallbacks::new();
		cb.credentials(|_, _, _| Cred::userpass_plaintext("oauth2", &input.source.private_token));

		let mut fo = FetchOptions::new();
		fo.remote_callbacks(cb);

		let mut builder = RepoBuilder::new();
		let repo = builder
			.fetch_options(fo)
			.branch(&mr.source_branch)
			.clone(&project.http_url_to_repo, Path::new(&args.directory))
			.with_context(|| anyhow!("failed to clone repository"))?;
		repo.reset(
			&repo.find_object(Oid::from_str(&mr.sha).unwrap(), None).unwrap(),
			git2::ResetType::Hard,
			None,
		)
		.with_context(|| anyhow!("failed to checkout {}", &mr.sha))?;
	}

	/* Dump version to a file for out */
	let file = File::create(Path::new(&args.directory).join(".merge-request.json"))
		.with_context(|| anyhow!("failed to create `.merge-request.json`"))?;
	serde_json::to_writer_pretty(file, &output.version)?;
	Ok(())
}

#[cfg(test)]
mod tests {
	use super::{
		Params,
		ResourceInput,
		Source,
	};
	use rstest::rstest;

	#[rstest]
	#[case::no_params(None, false)]
	#[case::skip_true(Some(Params { skip_clone: Some(true) }), true)]
	#[case::skip_false(Some(Params { skip_clone: Some(false) }), false)]
	#[case::no_skip_param(Some(Params { skip_clone: None }), false)]
	fn test_is_clone_skippable(#[case] params: Option<Params>, #[case] expect: bool) {
		let input = ResourceInput {
			params: params,
			source: Source {
				labels: None,
				paths: None,
				private_token: "".to_owned(),
				uri: "".to_owned(),
				skip_draft: None,
			},
			version: None,
		};
		assert_eq!(input.is_clone_skippable(), expect);
	}
}
