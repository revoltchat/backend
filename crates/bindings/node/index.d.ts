import { Channel, User } from "revolt-api";

/**
 * Opaque type for Revolt database
 */
export declare interface Database {}

/**
 * Opaque type for Revolt database
 */
export declare interface OpaqueUser {}

/**
 * Error type from Revolt backend
 */
export declare interface Err {
  type: string;
  location: string;
}

/**
 * Initialises background tasks and logging, must be called before anything else!
 * Can be called multiple times!
 */
export declare function init();

/**
 * Gets a new handle to the Revolt database
 * @returns {Database} Handle
 */
export declare function database(): Database;

/**
 * Fetch user from database
 * @param {string} userId User's ID
 * @this {Database}
 */
export declare function database_fetch_user(userId: string): OpaqueUser;

/**
 * Fetch user from database
 * @param {string} username Username
 * @param {string} discriminator Discriminator
 * @this {Database}
 */
export declare function database_fetch_user_by_username(
  username: string,
  discriminator: string
): OpaqueUser;

/**
 * Gets model data as JSON
 * @this {OpaqueUser}
 */
export declare function model_data(): User;

/**
 * Gets error if the model failed to fetch
 * @this {OpaqueUser}
 */
export declare function model_error(): Err;

/**
 * Open a direct message channel between two users
 * @param {string} userA User A ID
 * @param {string} userB User B ID
 * @returns Existing or newly created channel
 */
export declare function proc_channels_create_dm(
  userA: string,
  userB: string
): Promise<Channel & { error: Err }>;

/**
 * Suspend a user
 * @param {string} user User
 * @param {number} duration Duration (in days), set to 0 for indefinite
 * @param {string} reason Pipe-separated list of reasons (e.g. reason1|reason2|reason3)
 */
export declare function proc_users_suspend(
  user: OpaqueUser,
  duration: number,
  reason: string
): Promise<{ error: Err }>;
