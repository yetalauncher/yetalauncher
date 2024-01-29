use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};


#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AccountList {
    pub accounts: Vec<MCAccount>,
    pub selected_index: Option<u32>
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct MCAccount {
    pub msa_response: MSAResponse2,
    pub xbl_response: XBLResponse,
    pub xsts_response: XBLResponse,
    pub mc_response: MCResponse2,
    pub mc_profile: MCProfile
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct MCProfile {
    pub id: String,
    pub name: String,
    pub skins: Vec<MCSkin>,
    pub capes: Vec<MCCape>
}
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MCSkin {
    pub id: String,
    pub state: String,
    pub url: String,
    pub variant: String,
    pub texture_key: String,
    pub alias: Option<String>
}
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct MCCape {
    pub id: String
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct MSAResponse {
    pub token_type: String,
    pub expires_in: i32,
    pub scope: String,
    pub access_token: String,
    pub refresh_token: String,
    pub user_id: String
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct MSAResponse2 /* Electric Boogaloo! */ {
    pub token_type: String,
    pub expires_at: DateTime<Utc>,
    pub scope: String,
    pub access_token: String,
    pub refresh_token: String,
    pub user_id: String
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct XBLResponse {
    pub issue_instant: DateTime<Utc>,
    pub not_after: DateTime<Utc>,
    pub token: String,
    pub display_claims: XBLDisplayClaims
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct MCResponse {
    pub access_token: String,
    pub expires_in: i32,
    pub username: String,
    pub token_type: String
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct MCResponse2 /* Electric Boogaloo 2, Electric Boogaloo! ...ok not funny I get it */ {
    pub access_token: String,
    pub expires_at: DateTime<Utc>,
    pub username: String,
    pub token_type: String
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Entitlements {
    pub items: Vec<Entitlement>,
    pub signature: String
}
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Entitlement {
    pub name: String,
    pub source: String
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct XBLDisplayClaims {
    pub xui: Vec<XBLXUIClaims>
}
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct XBLXUIClaims {
    pub uhs: String
}