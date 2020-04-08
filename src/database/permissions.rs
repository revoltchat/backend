bitfield! {
    pub struct MemberPermissions(MSB0 [u8]);
    u8;
    pub get_access, set_access: 7;
    pub get_create_invite, set_create_invite: 6;
    pub get_kick_members, set_kick_members: 5;
    pub get_ban_members, set_ban_members: 4;
    pub get_read_messages, set_read_messages: 3;
    pub get_send_messages, set_send_messages: 2;
}

//struct PermissionCalculator {
    //channel: Option<>,
//}