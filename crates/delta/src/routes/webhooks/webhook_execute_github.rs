use revolt_quark::{Db, Ref, Result, Error, models::{Webhook, Message, message::SendableEmbed}, types::push::MessageAuthor};
use rocket::{Request, request::FromRequest, http::Status};
use rocket_okapi::{request::{OpenApiFromRequest, RequestHeaderInput}, okapi::openapi3::{Parameter, ParameterValue, MediaType}, gen::OpenApiGenerator};
use schemars::schema::{SchemaObject, StringValidation};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use ulid::Ulid;

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct GithubUser {
    name: Option<String>,
    email: Option<String>,
    login: String,
    id: u32,
    node_id: String,
    avatar_url: String,
    gravatar_id: String,
    url: String,
    html_url: String,
    followers_url: String,
    following_url: String,
    gists_url: String,
    starred_url: String,
    subscriptions_url: String,
    organizations_url: String,
    repos_url: String,
    events_url: String,
    received_events_url: String,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct GithubRepositorySecurityAndAnalysisStatus {
    status: String
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct GithubRepositorySecurityAndAnalysis {
    advanced_security: GithubRepositorySecurityAndAnalysisStatus,
    secret_scanning: GithubRepositorySecurityAndAnalysisStatus,
    secret_scanning_push_protection: GithubRepositorySecurityAndAnalysisStatus
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct GithubRepositoryLicense {
    key: Option<String>,
    name: Option<String>,
    spdx_id: Option<String>,
    url: Option<String>,
    node_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct GithubRepositoryCodeOfConduct {
    key: String,
    name: String,
    url: String,
    body: Option<String>,
    html_url: Option<String>
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct GithubRepositoryPermissions {
    admin: Option<bool>,
    maintain: Option<bool>,
    push: Option<bool>,
    triage: Option<bool>,
    pull: Option<bool>
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct GithubRepository {
    id: u32,
    node_id: String,
    name: String,
    full_name: String,
    owner: GithubUser,
    private: bool,
    html_url: String,
    description: Option<String>,
    fork: bool,
    url: String,
    archive_url: String,
    assignees_url: String,
    blobs_url: String,
    branches_url: String,
    collaborators_url: String,
    comments_url: String,
    commits_url: String,
    compare_url: String,
    contents_url: String,
    contributors_url: String,
    deployments_url: String,
    downloads_url: String,
    events_url: String,
    forks_url: String,
    git_commits_url: String,
    git_refs_url: String,
    git_tags_url: String,
    git_url: Option<String>,
    issue_comment_url: String,
    issue_events_url: String,
    issues_url: String,
    keys_url: String,
    labels_url: String,
    languages_url: String,
    merges_url: String,
    milestones_url: String,
    notifications_url: String,
    pulls_url: String,
    releases_url: String,
    ssh_url: String,
    stargazers_url: String,
    statuses_url: String,
    subscribers_url: String,
    subscription_url: String,
    tags_url: String,
    teams_url: String,
    trees_url: String,
    clone_url: Option<String>,
    mirror_url: Option<String>,
    hooks_url: String,
    svn_url: Option<String>,
    homepage: Option<String>,
    language: Option<String>,
    forks_count: Option<u32>,
    stargazers_count: Option<u32>,
    watchers_count: Option<u32>,
    size: Option<u64>,
    default_branch: Option<String>,
    open_issues_count: Option<String>,
    is_template: Option<bool>,
    topics: Option<Vec<String>>,
    has_issues: Option<bool>,
    has_projects: Option<bool>,
    has_wiki: Option<bool>,
    has_pages: Option<bool>,
    has_downloads: Option<bool>,
    has_discussions: Option<bool>,
    archived: Option<bool>,
    disabled: Option<bool>,
    visibility: Option<String>,
    pushed_at: Option<String>,
    created_at: Option<String>,
    updated_at: Option<String>,
    permissions: GithubRepositoryPermissions,
    role_name: Option<String>,
    temp_clone_token: Option<String>,
    delete_branch_on_merge: Option<bool>,
    subscribers_count: Option<u32>,
    network_count: Option<u32>,
    code_of_conduct: GithubRepositoryCodeOfConduct,
    license: Option<GithubRepositoryLicense>,
    forks: Option<u32>,
    open_issues: Option<u32>,
    watchers: Option<u32>,
    allow_forking: Option<bool>,
    web_commit_signoff_required: Option<bool>,
    security_and_analysis: Option<GithubRepositorySecurityAndAnalysis>
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum BaseEvent {
    Star {
        starred_at: Option<String>,
    }
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct Event {
    action: String,
    sender: GithubUser,
    repository: GithubRepository,
    organization: Option<Value>,
    installation: Option<Value>,

    #[serde(flatten)]
    event: BaseEvent
}

#[derive(Debug, JsonSchema)]
pub struct EventHeader<'r>(pub &'r str);

#[async_trait]
impl<'r> FromRequest<'r> for EventHeader<'r> {
    type Error = Error;

    async fn from_request(request: &'r Request<'_>) -> rocket::request::Outcome<Self,Self::Error> {
        let headers = request.headers();
        let Some(event) = headers.get_one("X-GitHub-Event") else {
            return rocket::request::Outcome::Failure((Status::BadRequest, Error::InvalidOperation))
        };

        rocket::request::Outcome::Success(Self(event))
    }
}

impl<'r> OpenApiFromRequest<'r> for EventHeader<'r> {
    fn from_request_input(
        _gen: &mut OpenApiGenerator,
        _name: String,
        _required: bool,
    ) -> rocket_okapi::Result<RequestHeaderInput> {
        let mut content = schemars::Map::new();
        content.insert("X-Github-Event".to_string(), MediaType {
            schema: Some(SchemaObject {
                string: Some(Box::new(StringValidation::default())),
                ..Default::default()
            }),
            example: None,
            examples: None,
            encoding: schemars::Map::new(),
            extensions: schemars::Map::new(),
        });

        Ok(RequestHeaderInput::Parameter(
            Parameter {
                name: "X-Github-Event".to_string(),
                location: "header".to_string(),
                required: true,
                description: Some("The name of the github event".to_string()),
                deprecated: false,
                allow_empty_value: false,
                value: ParameterValue::Content { content },
                extensions: schemars::Map::new()
            }
        ))
    }
}

/// # executes a webhook specific to github
///
/// executes a webhook specific to github and sends a message containg the relavent info about the event
#[openapi(tag = "Webhooks")]
#[post("/<target>/<token>/github", data="<data>")]
pub async fn req(db: &Db, target: Ref, token: String, event: EventHeader<'_>, data: String) -> Result<()> {
    let webhook = target.as_webhook(db).await?;

    (webhook.token == token)
        .then_some(())
        .ok_or(Error::InvalidCredentials)?;

    let channel = db.fetch_channel(&webhook.channel).await?;

    let body = format!(r#"{{"{}": {data}}}"#, &event.0);

    log::info!("{body}");

    let Ok(event) = serde_json::from_str::<Event>(&body) else {
        return Err(Error::InvalidOperation)
    };

    let sendable_embed = match event.event {
        BaseEvent::Star { .. } => {
            if event.action != "created" { return Ok(()) };

            SendableEmbed {
                title: Some(event.sender.login),
                description: Some(format!("[[{}] New star added]({})", event.repository.full_name, event.repository.html_url)),
                colour: Some("#202224".to_string()),
                icon_url: Some(event.sender.avatar_url),
                url: Some(event.sender.html_url),
                ..Default::default()
            }
        },
    };

    let message_id = Ulid::new().to_string();

    let embed = sendable_embed.into_embed(db, message_id.clone()).await?;

    let mut message = Message {
        id: message_id,
        channel: webhook.channel.clone(),
        embeds: Some(vec![embed]),
        webhook: Some(webhook.id.clone()),
        ..Default::default()
    };

    message.create(db, &channel, Some(MessageAuthor::Webhook(&webhook))).await
}
