use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum UserPermission {
    Read,
    Write,
    ReadAndWrite,
    Admin,
}

impl UserPermission {
    pub fn from_str(s: &str) -> Option<UserPermission> {
        match s {
            "read" => Some(UserPermission::Read),
            "write" => Some(UserPermission::Write),
            "read_and_write" => Some(UserPermission::ReadAndWrite),
            "admin" => Some(UserPermission::Admin),
            _ => None,
        }
    }

    pub fn from_i32(i: i32) -> Option<UserPermission> {
        match i {
            0 => Some(UserPermission::Read),
            1 => Some(UserPermission::Write),
            2 => Some(UserPermission::ReadAndWrite),
            3 => Some(UserPermission::Admin),
            _ => None,
        }
    }

    pub fn cmp(&self, other: &UserPermission) -> bool {
        match self {
            UserPermission::Read => matches!(other, UserPermission::Read),
            UserPermission::Write => matches!(other, UserPermission::Write),
            UserPermission::ReadAndWrite => matches!(
                other,
                UserPermission::Read | UserPermission::Write | UserPermission::ReadAndWrite
            ),
            UserPermission::Admin => true,
        }
    }
}
