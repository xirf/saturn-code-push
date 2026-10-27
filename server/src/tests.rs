use crate::{
    bootstrap::Runtime as Service,
    http::router,
    modules::{self, identity::api as auth},
    platform::{
        error::{Result, bad, internal},
        values::{now, uid},
    },
};
use axum::{
    body::Body,
    http::{StatusCode, header},
};
use axum::{http::Request, response::Response};
use http_body_util::BodyExt;
use modules::{
    applications::api::{App, Channel},
    artifacts::api::Artifact,
    distribution::api::{Check, Patch, Release},
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use tower::ServiceExt;

mod support;
use support::*;
mod artifacts;
mod distribution;
mod identity;
mod management;
mod migration;
mod security;
mod telemetry;

mod deployments;

mod accounts;

mod setup;
