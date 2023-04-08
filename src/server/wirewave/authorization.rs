use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum UserPermission {
    Read,
    Write,
    ReadAndWrite,
    Admin,
}

#[derive(Debug)]
pub enum UserPermissionError {
    UnknownPermission,
}

impl UserPermission {
    pub fn from_str(s: &str) -> Result<UserPermission, UserPermissionError> {
        match s {
            "read" => Ok(UserPermission::Read),
            "write" => Ok(UserPermission::Write),
            "read_and_write" => Ok(UserPermission::ReadAndWrite),
            "admin" => Ok(UserPermission::Admin),
            _ => Err(UserPermissionError::UnknownPermission),
        }
    }

    pub fn from_i32(i: i32) -> Result<UserPermission, UserPermissionError> {
        match i {
            0 => Ok(UserPermission::Read),
            1 => Ok(UserPermission::Write),
            2 => Ok(UserPermission::ReadAndWrite),
            3 => Ok(UserPermission::Admin),
            _ => Err(UserPermissionError::UnknownPermission),
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
