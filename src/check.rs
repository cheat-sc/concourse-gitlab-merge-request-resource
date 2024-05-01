mod common;
use anyhow::{
	anyhow,
	Result,
};
use chrono::{
	DateTime,
	Utc,
};
use common::*;
use gitlab::api::{
	common::{
		SortOrder,
		YesNo,
	},
	projects::{
		merge_requests::{
			self,
			MergeRequestOrderBy,
			MergeRequestState,
			MergeRequests,
		},
		repository::commits,
	},
	Query,
};
use gitlab::Gitlab;
use glob::Pattern;
use serde::Deserialize;
use std::io;
use std::str::FromStr;
use url::Url;

#[derive(Debug, Deserialize)]
pub struct ResourceInput {
	pub version: Option<Version>,
	pub source: Source,
}

fn main() -> Result<()> {
	let input: ResourceInput =
		get_data_from(&mut io::stdin()).map_err(|err| anyhow!("{}", err.downcast::<serde_json::Error>().unwrap()))?;

	let uri = Url::parse(&input.source.uri)?;
	let client = Gitlab::new(uri.host_str().unwrap(), &input.source.private_token)?;

	let mut builder = MergeRequests::builder();
	builder
		.project(uri.path().trim_start_matches('/').trim_end_matches(".git"))
		.order_by(MergeRequestOrderBy::UpdatedAt)
		.state(MergeRequestState::Opened)
		.sort(SortOrder::Ascending);

	/* filter mrs by updated date */
	if let Some(version) = &input.version {
		builder.updated_after(DateTime::<Utc>::from_str(&version.committed_date)?);
	}

	/* filter mrs by labels */
	if let Some(labels) = &input.source.labels {
		builder.labels(labels.iter());
	}

	/* filter mrs by draft */
	if let Some(skip_draft) = input.source.skip_draft {
		if skip_draft {
			builder.wip(YesNo::No);
		}
	}

	let mrs: Vec<MergeRequest> = builder.build()?.query(&client)?;

	let mut versions = Vec::<Version>::new();
	for mr in mrs {
		/* filter mrs by filepath in their changes */
		if let Some(paths) = &input.source.paths {
			let patterns: Vec<Pattern> = paths.iter().map(|path| Pattern::new(path).unwrap()).collect();

			let changes: MergeRequestChanges = merge_requests::MergeRequestChanges::builder()
				.project(uri.path().trim_start_matches('/').trim_end_matches(".git"))
				.merge_request(mr.iid)
				.build()?
				.query(&client)?;
			if !changes
				.changes
				.iter()
				.any(|change| patterns.iter().any(|pattern| pattern.matches(&change.new_path)))
			{
				continue;
			}
		}

		let commit: Commit = commits::Commit::builder()
			.project(mr.source_project_id)
			.commit(&mr.sha)
			.build()?
			.query(&client)?;
		versions.push(Version {
			iid: mr.iid.to_string(),
			committed_date: commit.committed_date,
			sha: mr.sha,
		});
	}

	println!("{}", serde_json::to_string_pretty(&versions)?);

	Ok(())
}
