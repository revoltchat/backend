use serde::{Deserialize, Serialize};

/// Reason for reporting content (message or server)
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
pub enum ContentReportReason {
    /// No reason has been specified
    NoneSpecified,

    /// Blatantly illegal content
    Illegal,

    /// Content that promotes harm to others / self
    PromotesHarm,

    /// Spam or platform abuse
    SpamAbuse,

    /// Distribution of malware
    Malware,

    /// Harassment or abuse targeted at another user
    Harassment,
}

/// Reason for reporting a user
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
pub enum UserReportReason {
    /// No reason has been specified
    NoneSpecified,

    /// User is sending spam or otherwise abusing the platform
    SpamAbuse,

    /// User's profile contains inappropriate content for a general audience
    InappropriateProfile,

    /// User is impersonating another user
    Impersonation,

    /// User is evading a ban
    BanEvasion,

    /// User is not of minimum age to use the platform
    Underage,
}

/// The content being reported
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
#[serde(tag = "type")]
pub enum ReportedContent {
    /// Report a message
    Message {
        /// ID of the message
        id: String,
        /// Reason for reporting message
        report_reason: ContentReportReason,
    },
    /// Report a server
    Server {
        /// ID of the server
        id: String,
        /// Reason for reporting server
        report_reason: ContentReportReason,
    },
    /// Report a user
    User {
        /// ID of the user
        id: String,
        /// Reason for reporting a user
        report_reason: UserReportReason,
    },
}

/// Status of the report
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
#[serde(tag = "status")]
pub enum ReportStatus {
    /// Report is waiting for triage / action
    Created {},

    /// Report was rejected
    Rejected { rejection_reason: String },

    /// Report was actioned and resolved
    Resolved {},
}

/// User-generated platform moderation report.
#[derive(Serialize, Deserialize, JsonSchema, Debug, OptionalStruct, Clone)]
#[optional_derive(Serialize, Deserialize, JsonSchema, Debug, Default, Clone)]
#[optional_name = "PartialReport"]
#[opt_skip_serializing_none]
pub struct Report {
    /// Unique Id
    #[serde(rename = "_id")]
    pub id: String,
    /// Id of the user creating this report
    pub author_id: String,
    /// Reported content
    pub content: ReportedContent,
    /// Additional report context
    pub additional_context: String,
    /// Status of the report
    #[opt_passthrough]
    #[serde(flatten)]
    pub status: ReportStatus,
    /// Additional notes included on the report
    #[serde(default)]
    pub notes: String,
}
