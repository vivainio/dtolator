use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Address {
    pub street: String,
    pub city: String,
    pub state: Option<String>,
    pub country: String,
    #[serde(rename = "postalCode")]
    pub postal_code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UserProfile {
    #[serde(rename = "firstName")]
    pub first_name: String,
    #[serde(rename = "lastName")]
    pub last_name: String,
    #[serde(rename = "phoneNumber")]
    pub phone_number: Option<String>,
    pub avatar: Option<String>,
    pub bio: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CreateUserRequest {
    pub email: String,
    pub name: String,
    pub age: Option<i64>,
    pub profile: UserProfile,
    pub address: Option<Address>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: i64,
    pub email: String,
    pub name: String,
    pub age: Option<i64>,
    #[serde(rename = "isActive")]
    pub is_active: Option<bool>,
    pub tags: Option<Vec<String>>,
    pub status: Option<String>,
    pub profile: Option<UserProfile>,
    pub address: Option<Address>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ApiResponse {
    pub success: bool,
    pub message: Option<String>,
    pub data: Option<User>,
}


