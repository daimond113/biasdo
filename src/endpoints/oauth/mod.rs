//! An implementation of OAuth 2.1 version 10 (draft-ietf-oauth-v2-1-10)
//! Hopes to be spec-compliant

use serde::{Deserialize, Serialize};
use std::{fmt::Display, str::FromStr};

pub mod authorization;
pub mod clients;
pub mod token;

fn is_true(b: &bool) -> bool {
	*b
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
	#[serde(skip_serializing_if = "is_true")]
	pub redirect: bool,
	pub error: &'static str,
	pub error_description: &'static str,
}

#[derive(Debug, Deserialize, Default)]
pub enum CodeChallengeMethod {
	#[default]
	#[serde(rename = "plain")]
	Plain,
	S256,
}

impl Display for CodeChallengeMethod {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			CodeChallengeMethod::Plain => f.write_str("plain"),
			CodeChallengeMethod::S256 => f.write_str("S256"),
		}
	}
}

impl FromStr for CodeChallengeMethod {
	type Err = String;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(match s {
			"plain" => CodeChallengeMethod::Plain,
			"S256" => CodeChallengeMethod::S256,
			_ => return Err(format!("Invalid code challenge method: {}", s)),
		})
	}
}
