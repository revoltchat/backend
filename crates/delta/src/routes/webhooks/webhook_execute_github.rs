use revolt_quark::{Db, Ref, Result, Error, models::{Message, message::SendableEmbed}, types::push::MessageAuthor};
use rocket::{Request, request::FromRequest, http::Status};
use revolt_rocket_okapi::{request::{OpenApiFromRequest, RequestHeaderInput}, revolt_okapi::openapi3::{Parameter, ParameterValue, MediaType}, gen::OpenApiGenerator};
use schemars::schema::SchemaObject;
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
    open_issues_count: Option<u32>,
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
    created_at: Value,
    updated_at: Option<String>,
    permissions: Option<GithubRepositoryPermissions>,
    role_name: Option<String>,
    temp_clone_token: Option<String>,
    delete_branch_on_merge: Option<bool>,
    subscribers_count: Option<u32>,
    network_count: Option<u32>,
    code_of_conduct: Option<GithubRepositoryCodeOfConduct>,
    license: Option<GithubRepositoryLicense>,
    forks: Option<u32>,
    open_issues: Option<u32>,
    watchers: Option<u32>,
    allow_forking: Option<bool>,
    web_commit_signoff_required: Option<bool>,
    security_and_analysis: Option<GithubRepositorySecurityAndAnalysis>
}


#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct CommitAuthor {
    date: Option<String>,
    email: Option<String>,
    name: String,
    username: Option<String>
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct GithubCommit {
    added: Option<Vec<String>>,
    author: CommitAuthor,
    committer: CommitAuthor,
    distinct: bool,
    id: String,
    message: String,
    modified: Option<Vec<String>>,
    removed: Option<Vec<String>>,
    timestamp: String,
    tree_id: String,
    url: String
}
#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct GithubReactions {
    #[serde(rename="+1")]
    plus_one: u32,
    #[serde(rename="-1")]
    minus_one: u32,
    confused: u32,
    eyes: u32,
    heart: u32,
    hooray: u32,
    laugh: u32,
    rocket: u32,
    total_count: u32,
    url: String
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct GithubComment {
    author_association: String,
    body: String,
    commit_id: String,
    created_at: Value,
    html_url: String,
    id: u32,
    line: Option<u32>,
    node_id: String,
    path: Option<String>,
    position: Option<u32>,
    reactions: Option<GithubReactions>,
    updated_at: String,
    url: String,
    user: GithubUser
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct GithubDiscussionComment {
    #[serde(flatten)]
    comment: GithubComment,
    child_comment_count: u32,
    parent_id: Option<u32>,
    repository_url: String
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct GithubDiscussionCategory {
    id: u32,
    node_id: String,
    repository_id: u32,
    emoji: String,
    name: String,
    description: String,
    created_at: Value,
    updated_at: String,
    slug: String,
    is_answerable: bool
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct GithubDiscussion {
    repository_url: String,
    category: GithubDiscussionCategory,
    answer_html_url: Option<String>,
    answer_chosen_at: Value,  // ??
    answer_chosen_by: Value,  // ??
    html_url: String,
    id: u32,
    node_id: String,
    number: u32,
    title: String,
    user: GithubUser,
    state: String,
    locked: bool,
    comments: u32,
    created_at: Value,
    updated_at: String,
    author_association: String,
    active_lock_reason: Option<String>,
    body: String,
    reactions: GithubReactions,
    timeline_url: String
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "action")]
#[allow(clippy::large_enum_variant)]
pub enum GithubDiscussionEvent {
    Created {
        discussion: GithubDiscussion
    },
    Answered {
        discussion: GithubDiscussion,
        answer: GithubDiscussionComment
    }
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "action")]
#[allow(clippy::large_enum_variant)]
pub enum DiscussionCommentEvent {
    Created {
        comment: GithubDiscussionComment,
        discussion: GithubDiscussion
    },
    Deleted {},
    Edited {}
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct GithubMilestone {
    url: String,
    html_url: String,
    labels_url: String,
    id: u32,
    node_id: String,
    number: u32,
    state: String,
    title: String,
    description: String,
    creator: Option<GithubUser>,
    open_issues: u32,
    closed_issues: u32,
    created_at: Value,
    updated_at: String,
    closed_at: Option<String>,
    due_on: Option<String>
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct GithubLinkedPullRequest {
    merged_at: Option<String>,
    diff_url: Option<String>,
    html_url: Option<String>,
    patch_url: Option<String>,
    url: Option<String>,
}
#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct GithubAppPermissions {
    issues: Option<String>,
    checks: Option<String>,
    metadata: Option<String>,
    contents: Option<String>,
    deployments: Option<String>
}


#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct GithubApp {
    id: u32,
    slug: String,
    node_is: String,
    owner: Option<GithubUser>,
    name: String,
    description: Option<String>,
    external_url: String,
    html_url: String,
    created_at: Value,
    updated_at: String,
    permissions: GithubAppPermissions,
    events: Vec<String>,
    installations_count: Option<u32>,
    client_id: Option<String>,
    client_secret: Option<String>,
    webhook_secret: Option<String>,
    pem: Option<String>
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct GithubIssue {
    id: u32,
    node_id: String,
    url: String,
    repository_url: String,
    labels_url: String,
    comments_url: String,
    events_url: String,
    html_url: String,
    number: u32,
    state: String,
    state_reason: Option<String>,
    user: Option<GithubUser>,
    labels: Vec<String>,
    assignee: Option<GithubUser>,
    assignees: Option<Vec<GithubUser>>,
    milestone: Option<GithubMilestone>,
    locked: bool,
    active_lock_reason: Option<String>,
    comments: u32,
    pull_request: Option<GithubLinkedPullRequest>,
    closed_at: Option<String>,
    created_at: Value,
    updated_at: String,
    draft: Option<bool>,
    closed_by: Option<GithubUser>,
    body_html: Option<String>,
    body_text: Option<String>,
    timeline_url: Option<String>,
    repository: Option<GithubRepository>,
    performed_via_github_app: Option<GithubApp>,
    author_accociation: String,
    reactions: Option<GithubReactions>,
    title: String
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "action")]
#[allow(clippy::large_enum_variant)]
pub enum IssueCommentEvent {
    Created {
        comment: GithubComment,
        issue: GithubIssue
    },
    Deleted {},
    Edited {}
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "action")]
#[allow(clippy::large_enum_variant)]
pub enum IssuesEvent {
    Assigned {},
    Closed {
        issue: GithubIssue
    },
    Deleted {},
    Demilestoned {},
    Edited {},
    Labeled {},
    Locked {},
    Milestoned {},
    Opened {
        issue: GithubIssue
    },
    Pinned {},
    Reopened {
        issue: GithubIssue
    },
    Transferred {},
    Unassigned {},
    Unlabeled {},
    Unlocked {},
    Unpinned {}
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "type")]
#[allow(clippy::large_enum_variant)]
pub enum BaseEvent {
    Star {
        starred_at: Option<String>,
    },
    Ping {},
    Push {
        after: String,
        base_ref: Option<String>,
        before: String,
        commits: Vec<GithubCommit>,
        compare: String,
        created: bool,
        deleted: bool,
        forced: bool,
        head_commit: Option<GithubCommit>,
        pusher: CommitAuthor,
        r#ref: String
    },
    CommitComment {
        comment: GithubComment
    },
    Create {
        master_branch: String,
        pusher_type: String,
        r#ref: String,
        ref_type: String
    },
    Delete {
        pusher_type: String,
        r#ref: String,
        ref_type: String
    },
    Discussion(GithubDiscussionEvent),
    DiscussionComment(DiscussionCommentEvent),
    Fork {
        forkee: GithubRepository
    },
    IssueComment(IssueCommentEvent),
    Issues(IssuesEvent)

}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct Event {
    action: Option<String>,
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
    ) -> revolt_rocket_okapi::Result<RequestHeaderInput> {
        let mut content = schemars::Map::new();
        content.insert("X-Github-Event".to_string(), MediaType {
            schema: Some(SchemaObject {
                string: Some(Box::default()),
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

const GREEN: &str = "#0f890f";
const RED: &str = "#e73e3e";
const GREY: &str = "#202224";
const BLUE: &str = "#279adc";
const ORANGE: &str = "#d76a34";
const LIGHT_ORANGE: &str = "#d9916d";
const WHITE: &str = "#c3e1c3";

fn shorten_text(text: &str, length: usize) -> String {
    if text.len() < length {
        text.to_string()
    } else {
        format!("{}...", &text[0..length])
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

    let body = format!(r#"{}, "type": "{}"}}"#, &data[0..data.len() - 1], &event.0);

    log::info!("{body}");

    let event = match serde_json::from_str::<Event>(&body) {
        Ok(event) => event,
        Err(err) => {
            log::error!("{err:?}");
            return Err(Error::InvalidOperation);
        }
    };

    log::info!("{event:?}");

    let sendable_embed = match event.event {
        BaseEvent::Star { .. } => {
            if event.action.as_deref() != Some("created") { return Ok(()) };

            SendableEmbed {
                title: Some(event.sender.login),
                description: Some(format!("#### [[{}] New star added]({})", event.repository.full_name, event.repository.html_url)),
                colour: Some(GREY.to_string()),
                icon_url: Some(event.sender.avatar_url),
                url: Some(event.sender.html_url),
                ..Default::default()
            }
        },
        BaseEvent::Push { after, commits, compare, forced, r#ref, .. } => {
            let Some(branch) = r#ref.split('/').nth(2) else { return Ok(()) };

            if forced {
                let description = format!("#### [{}] Branch {} was force-pushed to {}\n[compare changes]({})", event.repository.full_name, branch, &after[0..=7], compare);

                SendableEmbed {
                    icon_url: Some(event.sender.avatar_url),
                    title: Some(event.sender.login),
                    description: Some(description),
                    url: Some(event.sender.html_url),
                    colour: Some(RED.to_string()),
                    ..Default::default()
                }
            } else {
                let title = format!("[[{}:{}] {} new commit]({})", event.repository.full_name, branch, commits.len(), compare);
                let commit_description = commits
                    .into_iter()
                    .map(|commit| format!("[`{}`]({}) {} - {}", &commit.id[0..=7], commit.url, shorten_text(&commit.message, 50), commit.author.name))
                    .collect::<Vec<String>>()
                    .join("\n");

                SendableEmbed {
                    title: Some(event.sender.login),
                    description: Some(format!("#### {title}\n{commit_description}")),
                    colour: Some(BLUE.to_string()),
                    icon_url: Some(event.sender.avatar_url),
                    url: Some(event.sender.html_url),
                    ..Default::default()
                }
            }
        },
        BaseEvent::CommitComment { comment } => {
            SendableEmbed {
                icon_url: Some(event.sender.avatar_url),
                url: Some(event.sender.html_url),
                title: Some(event.sender.login),
                description: Some(format!("#### [[{}] New comment on commit `{}`]({})\n{}", event.repository.full_name, &comment.commit_id[0..=7], comment.html_url, shorten_text(&comment.body, 450))),
                colour: Some(GREY.to_string()),
                ..Default::default()
            }
        },
        BaseEvent::Ping {  } => {
            return Ok(())
        },
        BaseEvent::Create { r#ref, ref_type, .. } => {
            SendableEmbed {
                icon_url: Some(event.sender.avatar_url),
                url: Some(event.sender.html_url),
                title: Some(event.sender.login),
                description: Some(format!("#### [{}] New {} created: {}", event.repository.full_name, ref_type, r#ref)),
                colour: Some(GREY.to_string()),
                ..Default::default()
            }
        },
        BaseEvent::Delete { r#ref, ref_type, .. } => {
            SendableEmbed {
                icon_url: Some(event.sender.avatar_url),
                url: Some(event.sender.html_url),
                title: Some(event.sender.login),
                description: Some(format!("#### [{}] {} deleted: {}", event.repository.full_name, ref_type, r#ref)),
                colour: Some(GREY.to_string()),
                ..Default::default()
            }
        },
        BaseEvent::Discussion(discussion_event) => {
            match discussion_event {
                GithubDiscussionEvent::Created { discussion } => {
                    SendableEmbed {
                        icon_url: Some(event.sender.avatar_url),
                        url: Some(event.sender.html_url),
                        title: Some(event.sender.login),
                        description: Some(format!("#### [{}] New discussion #{}: {}\n{}", event.repository.full_name, discussion.number, discussion.title, shorten_text(&discussion.body, 450))),
                        colour: Some(LIGHT_ORANGE.to_string()),
                        ..Default::default()
                    }
                },
                GithubDiscussionEvent::Answered { discussion, answer } => {
                    SendableEmbed {
                        icon_url: Some(answer.comment.user.avatar_url),
                        url: Some(answer.comment.user.html_url),
                        title: Some(answer.comment.user.login),
                        description: Some(format!("#### [{}] discussion #{} marked answered: {}\n{}", event.repository.full_name, discussion.number, discussion.title, shorten_text(&answer.comment.body, 450))),
                        colour: Some(LIGHT_ORANGE.to_string()),
                        ..Default::default()
                    }
                },
            }
        },
        BaseEvent::DiscussionComment(comment_event) => {
            match comment_event {
                DiscussionCommentEvent::Created { comment, discussion } => {
                    SendableEmbed {
                        icon_url: Some(comment.comment.user.avatar_url),
                        url: Some(comment.comment.user.html_url),
                        title: Some(comment.comment.user.login),
                        description: Some(format!("[{}] New comment on discussion #{}: {}\n{}", event.repository.full_name, discussion.number, discussion.title, shorten_text(&comment.comment.body, 450))),
                        colour: Some(LIGHT_ORANGE.to_string()),
                        ..Default::default()
                    }
                },
                _ => { return Ok(()) }
            }
        },
        BaseEvent::Fork { forkee } => {
            SendableEmbed {
                icon_url: Some(event.sender.avatar_url),
                url: Some(event.sender.html_url),
                title: Some(event.sender.login),
                description: Some(format!("#### [[{}] Fork created: {}]({})", event.repository.full_name, forkee.full_name, forkee.html_url)),
                colour: Some(GREY.to_string()),
                ..Default::default()
            }
        },
        BaseEvent::Issues(issue_event) => {
            match issue_event {
                IssuesEvent::Closed { issue } => {
                    SendableEmbed {
                        icon_url: Some(event.sender.avatar_url),
                        url: Some(event.sender.html_url),
                        title: Some(event.sender.login),
                        description: Some(format!("#### [[{}] Issue Closed #{}: {}]({})", event.repository.full_name, issue.number, issue.title, issue.html_url)),
                        colour: Some(GREY.to_string()),
                        ..Default::default()
                    }
                },
                IssuesEvent::Opened { issue } => {
                    SendableEmbed {
                        icon_url: Some(event.sender.avatar_url),
                        url: Some(event.sender.html_url),
                        title: Some(event.sender.login),
                        description: Some(format!("#### [[{}] Issue Opened #{}: {}]({})\n{}", event.repository.full_name, issue.number, issue.title, issue.html_url, issue.body_text.unwrap_or_default())),
                        colour: Some(ORANGE.to_string()),
                        ..Default::default()
                    }
                },
                IssuesEvent::Reopened { issue } => {
                    SendableEmbed {
                        icon_url: Some(event.sender.avatar_url),
                        url: Some(event.sender.html_url),
                        title: Some(event.sender.login),
                        description: Some(format!("#### [[{}] Issue Reopened #{}: {}]({})", event.repository.full_name, issue.number, issue.html_url, issue.title)),
                        colour: Some(GREEN.to_string()),
                        ..Default::default()
                    }
                },
                _ => { return Ok(()) }
            }
        },
        BaseEvent::IssueComment(comment_event) => {
            match comment_event {
                IssueCommentEvent::Created { comment, issue } => {
                    SendableEmbed {
                        icon_url: Some(event.sender.avatar_url),
                        url: Some(event.sender.html_url),
                        title: Some(event.sender.login),
                        description: Some(format!("#### [[{}] New comment on issue #{}: {}]({})\n{}", event.repository.full_name, issue.number, issue.title, issue.html_url, comment.body)),
                        colour: Some(LIGHT_ORANGE.to_string()),
                        ..Default::default()
                    }
                },
                _ => { return Ok(()) }
            }
        }
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
