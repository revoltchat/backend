use iso8601_timestamp::Timestamp;

auto_derived!(
    /// User-generated platform moderation report
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
        #[serde(flatten)]
        pub status: ReportStatus,
        /// Additional notes included on the report
        #[serde(default)]
        pub notes: String,
    }

    /// Reason for reporting content (message or server)
    pub enum ContentReportReason {
        /// No reason has been specified
        NoneSpecified,

        /// Illegal content catch-all reason
        Illegal,

        /// Selling or facilitating use of drugs or other illegal goods
        IllegalGoods,

        /// Extortion or blackmail
        IllegalExtortion,

        /// Revenge or child pornography
        IllegalPornography,

        /// Illegal hacking activity
        IllegalHacking,

        /// Extreme violence, gore, or animal cruelty
        /// With exception to violence potrayed in media / creative arts
        ExtremeViolence,

        /// Content that promotes harm to others / self
        PromotesHarm,

        /// Unsolicited advertisements
        UnsolicitedSpam,

        /// This is a raid
        Raid,

        /// Spam or platform abuse
        SpamAbuse,

        /// Scams or fraud
        ScamsFraud,

        /// Distribution of malware or malicious links
        Malware,

        /// Harassment or abuse targeted at another user
        Harassment,
    }

    /// Reason for reporting a user
    pub enum UserReportReason {
        /// No reason has been specified
        NoneSpecified,

        /// Unsolicited advertisements
        UnsolicitedSpam,

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
            /// Message context
            message_id: Option<String>,
        },
    }

    /// Status of the report
    #[serde(tag = "status")]
    pub enum ReportStatus {
        /// Report is waiting for triage / action
        Created {},

        /// Report was rejected
        Rejected {
            rejection_reason: String,
            closed_at: Option<Timestamp>,
        },

        /// Report was actioned and resolved
        Resolved { closed_at: Option<Timestamp> },
    }

    /// Just the status of the report
    pub enum ReportStatusString {
        /// Report is waiting for triage / action
        Created,

        /// Report was rejected
        Rejected,

        /// Report was actioned and resolved
        Resolved,
    }
);
