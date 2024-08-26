const Internal = require(".");

// playing around with class wrapper, not practical
class Model {
  constructor(model) {
    this.model = model;
  }

  data() {
    return Internal.model_data.bind(this.model)();
  }

  error() {
    return Internal.model_error.bind(this.model)();
  }
}

class User extends Model {
  constructor(db, user) {
    super(user);
    this.db = db;
  }
}

class Database {
  constructor() {
    this.db = Internal.database();
  }

  async fetchUser(userId) {
    return new User(
      this,
      await Internal.database_fetch_user.bind(this.db)(userId)
    );
  }

  async fetchUserByUsername(username, discriminator) {
    return new User(
      this,
      await Internal.database_fetch_user_by_username.bind(this.db)(
        username,
        discriminator
      )
    );
  }
}

const db = new Database();
db.fetchUserByUsername("dos", "7624").then((user) => console.info(user.data()));
db.fetchUserByUsername("dos", "1111").then((user) => console.info(user.data()));
db.fetchUserByUsername("dos", "1111").then((user) =>
  console.info(user.error())
);
