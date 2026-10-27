use postgres::Transaction as Connection;
use serde_json::{Value, json};

use super::{
    models::{App, Channel},
    repositories::{app_repository, channel_repository},
};
use crate::platform::{
    database::next_id,
    error::{Result, conflict},
    values::{now, text, uid},
};

fn app_json(app: &App) -> Value {
    json!({
        "id": app.id,
        "app_id": app.id,
        "organization_id": 1,
        "display_name": app.display_name,
        "created_at": app.created_at,
        "updated_at": app.created_at,
    })
}

pub(super) fn list_apps(connection: &mut Connection<'_>, scope: Option<String>) -> Result<Value> {
    let apps = app_repository::list(connection)?;
    let apps: Vec<Value> = apps
        .iter()
        .filter(|a| scope.as_ref().is_none_or(|id| &a.id == id))
        .map(app_json)
        .collect();
    Ok(json!({"apps": apps}))
}

pub(super) fn get_app(connection: &mut Connection<'_>, app_id: String) -> Result<Value> {
    let app = app_repository::find(connection, &app_id)?;
    Ok(app_json(&app))
}

pub(super) fn create_app(connection: &mut Connection<'_>, input: Value) -> Result<Value> {
    let app = App {
        id: uid(),
        display_name: text(&input, "display_name")?,
        created_at: now(),
    };
    app_repository::save(connection, &app)?;
    for name in ["stable", "beta", "staging"] {
        let channel = Channel {
            id: next_id(connection)?,
            app_id: app.id.clone(),
            name: name.into(),
        };
        channel_repository::save(connection, &channel)?;
    }
    Ok(app_json(&app))
}

pub(super) fn list_channels(connection: &mut Connection<'_>, app_id: String) -> Result<Value> {
    app_repository::find(connection, &app_id)?;
    let channels: Vec<Channel> = channel_repository::list(connection)?
        .into_iter()
        .filter(|channel| channel.app_id == app_id)
        .collect();
    Ok(json!(channels))
}

pub(super) fn create_channel(
    connection: &mut Connection<'_>,
    app_id: String,
    input: Value,
) -> Result<Value> {
    app_repository::find(connection, &app_id)?;
    let name = text(&input, "channel")?;
    let exists = channel_repository::list(connection)?
        .iter()
        .any(|channel| channel.app_id == app_id && channel.name == name);
    if exists {
        return Err(conflict("Channel exists"));
    }
    let channel = Channel {
        id: next_id(connection)?,
        app_id,
        name,
    };
    channel_repository::save(connection, &channel)?;
    Ok(json!(channel))
}
