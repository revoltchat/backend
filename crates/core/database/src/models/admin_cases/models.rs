auto_derived_partial! {
    pub struct AdminCase {
        /// The case ID
        #[serde(rename = "_id")]
        pub id: String,
        /// The case Short ID
        pub short_id: String,

        /// The owner of the case
        pub owner_id: String,
        /// The title of the case
        pub title: String,
        /// The status of the case (open/closed)
        pub status: String,
        /// When the case was closed, in iso8601
        pub closed_at: Option<String>,
        /// The tags for the case
        pub tags: Vec<String>,
    },
    "PartialAdminCase"
}

impl AdminCase {
    pub fn new(owner_id: &str, title: &str) -> AdminCase {
        let id = ulid::Ulid::new().to_string();
        let short_id = id.clone().split_off(id.len() - 7);

        AdminCase {
            id,
            short_id,
            owner_id: owner_id.to_string(),
            title: title.to_string(),
            status: "Open".to_string(),
            closed_at: None,
            tags: vec![],
        }
    }

    pub fn merge_tags(&self, other: &[String]) -> Vec<String> {
        let mut resp: Vec<String> = vec![];
        // Shitty combining chain; itll only ever be like 5 items
        resp.extend(self.tags.clone());
        resp.extend(other.iter().filter_map(|p| {
            if !self.tags.contains(p) {
                Some(p.clone())
            } else {
                None
            }
        }));

        resp
    }
}
