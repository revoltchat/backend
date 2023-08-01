# Code Style Guide

Beyond using Cargo format and Clippy, there are some specific code style guidelines laid out in this document for different parts of the project.

## Writing Style

- Shorten "identifier" to "Id" with that exact casing, i.e. Server Id.

## `core/database` crate

w.r.t. `model.rs` files

- All struct definitions must be commented.
  ```rust
  /// Server
  pub struct Server {
    /// Name of the server
    pub name: String,
  ```
- Struct definitions should not include derives unless necessary (if additional traits such as Hash are required) and instead use `auto_derived!` and `auto_derived_partial!`.
  ```rust
  auto_derived_partial!(
    /// Server
    pub struct Server { .. },
    "PartialServer"
  );
  ```
- `auto_derived!` macro accepts multiple entries and should be used as such:

  ```rust
  auto_derived!(
    /// Optional fields on server object
    pub enum FieldsServer { .. }

    /// Optional fields on server object
    pub enum FieldsRole { .. }
  );
  ```

- If special serialisation conditions are required, such as checking if a boolean is false, use the existing definitions for these functions from the crate root:
  ```rust
  #[serde(skip_serializing_if = "crate::if_false", default)]
  ```
- `impl` blocks may be defined below the struct definitions and should be ordered in the same order of definition. Methods in the block must follow the same guidelines as traits where-in: methods are ordered in terms of CRUD, there are empty line breaks, and methods are commented.

w.r.t. `ops` module for models

- All traits must use a the name format `AbstractPlural` where Plural is the plural form of the collection. e.g. Servers
- Traits defined must follow these guidelines:

  - Methods are ordered in terms of CRUD, create-read-update-delete ordering.

    ```rust
    #[async_trait]
    pub trait AbstractServerMembers: Sync + Send {
      /// Insert a new server member into the database
      async fn insert_member(&self, member: &Member) -> Result<()>;

      /// Fetch a server member by their id
      async fn fetch_member(&self, server_id: &str, user_id: &str) -> Result<Member>;

      /// Update information for a server member
      async fn update_member(&self, .. ) -> Result<()>;

      /// Delete a server member by their id
      async fn delete_member(&self, id: &MemberCompositeKey) -> Result<()>;
    }
    ```

  - There should be an empty line break between each method declaration.
  - All methods must have an appropriate comment.

- When implementing the trait defined in `ops.rs` with each driver, the method declaration style should be the same for ease of searching: same ordering, same comments, same line breaks.
