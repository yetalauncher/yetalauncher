use afire::{Server, Method, Response, Status};
use chrono::{Utc, Duration};
use log::*;
use reqwest::Client;
use serde_json::json;
use tokio::runtime::Handle;
use uuid::Uuid;

use crate::{app::{accounts::{save_new_account, update_account}, consts::{MS_CLIENT_ID, REDIRECT_PORT}, utils::{NotificationState, Notifier}}, launcher::authentication::auth_structs::*};

pub mod auth_structs;


#[allow(dead_code)] // for now, until this is reimplemented
fn get_login_url() -> String {
    String::from_iter([
        "https://login.live.com/oauth20_authorize.srf?client_id=",
        MS_CLIENT_ID,
        "&prompt=select_account",
        "&cobrandid=8058f65d-ce06-4c30-9559-473c9275a65d",
        "&response_type=code",
        "&scope=XboxLive.signin%20XboxLive.offline_access",
        "&redirect_uri=http%3A%2F%2F127.0.0.1%3A",
        &REDIRECT_PORT.to_string()
    ])
}
fn get_msa_url() -> String {
    String::from("https://login.live.com/oauth20_token.srf")
}
fn get_xbl_url() -> String {
    String::from("https://user.auth.xboxlive.com/user/authenticate")
}
fn get_xsts_url() -> String {
    String::from("https://xsts.auth.xboxlive.com/xsts/authorize")
}
fn get_mc_url() -> String {
    String::from("https://api.minecraftservices.com/launcher/login")
}
fn get_entitlements_url() -> String {
    String::from("https://api.minecraftservices.com/entitlements/license")
}
fn get_mc_profile_url() -> String {
    String::from("https://api.minecraftservices.com/minecraft/profile")
}



pub async fn add_account() {
    let mut redirect_server = Server::<()>::new("127.0.0.1", REDIRECT_PORT);

    let notifier = Notifier::new("login_status");

    notifier.notify("Awaiting login", NotificationState::Running);
    tokio::task::spawn(async move {
        redirect_server.route(Method::GET, "/", move |req| {
            if let Some(code) = req.query.get("code") {
                info!("Code obtained!");
                notifier.notify("Beginning login process...", NotificationState::Running);
                Handle::current().block_on(add_account_code(code, &notifier));
                Response::new()
                    .text("You may close this window now.")
                    .status(Status::Ok)
            } else {
                error!("Getting Code failed!");
                notifier.notify("Failed getting code from response!", NotificationState::Error);
                Response::new()
                    .text("Failed to get the authentication code!")
                    .status(Status::NotFound)
            }
        });

        info!("Starting auth redirect HTTP server on port {REDIRECT_PORT}...");
        if let Err(e) = redirect_server.start() {
            error!("Starting redirect server failed: {e}")
        };
    }).await.unwrap();
}

async fn add_account_code(code: &str, notifier: &Notifier) {
    info!("Started adding new Minecraft account!");
    let client = Client::new();

    info!("Getting Microsoft Auth response...");
    notifier.notify("Getting Microsoft Auth reponse...", NotificationState::Running);
    let msa_response = MSAResponse2::from_code(code, &client).await;
    // trace!("{:#?}", msa_response);

    info!("Getting Xbox Live Auth response...");
    notifier.notify("Getting Xbox Live Auth reponse...", NotificationState::Running);
    let xbl_response = msa_response.get_xbl_reponse(&client).await;
    // trace!("{:#?}", xbl_response);

    info!("Getting Xsts Auth response...");
    notifier.notify("Getting Xsts Auth reponse...", NotificationState::Running);
    let xsts_response = xbl_response.xbl_to_xsts_response(&client).await;
    // trace!("{:#?}", xsts_response);

    info!("Getting Minecraft Auth response...");
    notifier.notify("Getting Minecraft Auth reponse...", NotificationState::Running);
    let mc_response = xsts_response.xsts_to_mc_response(&client).await;
    // trace!("{:#?}", mc_response);

    info!("Checking Minecraft ownership...");
    notifier.notify("Checking Minecraft ownership...", NotificationState::Running);
    if !mc_response.has_mc_ownership(&client).await {
        notifier.notify("Account does not own Minecraft!", NotificationState::Error);
        return;
    }

    info!("Getting Minecraft account...");
    notifier.notify("Getting Minecraft account...", NotificationState::Running);
    let mc_profile = mc_response.get_mc_profile(&client).await;
    // trace!("{:#?}", mc_profile);

    let mc_account = MCAccount {
        msa_response,
        xbl_response,
        xsts_response,
        mc_response,
        mc_profile
    };
    let username = mc_account.mc_profile.name.clone();

    // trace!("{:#?}", mc_account);
    info!("Saving new Minecraft account...");
    notifier.notify("Saving new account...", NotificationState::Running);
    if let Err(e) = save_new_account(mc_account) {
        error!("Error occured while saving new account: {e}")
    }

    notifier.notify(&String::from_iter(["Successfully added account \"", &username, "\"!"]), NotificationState::Success);
    info!("Successfully added new account.");
}



impl MSAResponse2 {
    async fn from_code(code: &str, client: &Client) -> Self {
        let params = [
            ("client_id", MS_CLIENT_ID),
            ("code", code),
            ("grant_type", "authorization_code"),
            ("redirect_uri", &String::from_iter(["http://127.0.0.1:", &REDIRECT_PORT.to_string()])),
            ("scope", "XboxLive.signin XboxLive.offline_access")
        ];

        let msa_response: MSAResponse = client.post(get_msa_url())
        .form(&params)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

        MSAResponse2 {
            token_type: msa_response.token_type,
            expires_at: Utc::now() + Duration::seconds(msa_response.expires_in.into()),
            scope: msa_response.scope,
            access_token: msa_response.access_token,
            refresh_token: msa_response.refresh_token,
            user_id: msa_response.user_id,
        }
    }

    async fn refresh(&mut self, client: &Client) {
        let params = [
            ("client_id", MS_CLIENT_ID),
            ("grant_type", "refresh_token"),
            ("refresh_token", &self.refresh_token),
            ("scope", &self.scope)
        ];

        let msa_response: MSAResponse = client.post(get_msa_url())
        .form(&params)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

        debug!("res {:#?}", msa_response);

        *self = MSAResponse2 {
            token_type: msa_response.token_type,
            expires_at: Utc::now() + Duration::seconds(msa_response.expires_in.into()),
            scope: msa_response.scope,
            access_token: msa_response.access_token,
            refresh_token: msa_response.refresh_token,
            user_id: msa_response.user_id,
        };
    }

    async fn get_xbl_reponse(&self, client: &Client) -> XBLResponse {
        let json = json!({
            "Properties": {
                "AuthMethod": "RPS",
                "SiteName": "user.auth.xboxlive.com",
                "RpsTicket": &String::from_iter(["d=", &self.access_token])
            },
            "RelyingParty": "http://auth.xboxlive.com",
            "TokenType": "JWT"
        });

        client.post(get_xbl_url())
        .json(&json)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap()
    }
}

impl XBLResponse {
    async fn xbl_to_xsts_response(&self, client: &Client) -> Self {
        let json = json!({
            "Properties": {
                "SandboxId": "RETAIL",
                "UserTokens": [
                    self.token
                ]
            },
            "RelyingParty": "rp://api.minecraftservices.com/",
            "TokenType": "JWT"
        });

        client.post(get_xsts_url())
        .json(&json)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap()
    }

    async fn xsts_to_mc_response(&self, client: &Client) -> MCResponse2 {
        let json = json!({
            "xtoken": String::from_iter(["XBL3.0 x=", &self.display_claims.xui[0].uhs, ";", &self.token]),
            "platform": "PC_LAUNCHER"
        });
    
        let mc_response: MCResponse = client.post(get_mc_url())
        .json(&json)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    
        MCResponse2 {
            access_token: mc_response.access_token,
            expires_at: Utc::now() + Duration::seconds(mc_response.expires_in.into()),
            username: mc_response.username,
            token_type: mc_response.token_type,
        }
    }
}

impl MCResponse2 {
    async fn get_mc_profile(&self, client: &Client) -> MCProfile {
        let mcprofile_response: MCProfile = client.get(get_mc_profile_url())
        .header("Authorization", String::from_iter(["Bearer ", &self.access_token]))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    
        mcprofile_response
    }

    async fn has_mc_ownership(&self, client: &Client) -> bool {
        let entitlements_response: Entitlements = client.get(
            String::from_iter([&get_entitlements_url(), "?requestId=", &Uuid::new_v4().to_string()])
        )
        .header("Authorization", String::from_iter(["Bearer ", &self.access_token]))
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
        // trace!("{:#?}", entitlements_response);

        entitlements_response.items.iter().any(|item| 
            item.name.eq_ignore_ascii_case("product_minecraft") || item.name.eq_ignore_ascii_case("game_minecraft")
        )
    }
}


impl MCAccount {
    pub async fn refresh(&mut self, client: &Client, force: bool) {
        let previous = self.clone();
        let now = Utc::now();

        if self.mc_response.expires_at < now || force {
            if self.xsts_response.not_after < now || force {
                if self.xbl_response.not_after < now || force {
                    if self.msa_response.expires_at < now || force {
                        info!("Refreshing Microsoft Token...");
                        self.msa_response.refresh(client).await;
                    }
                    info!("Refreshing Xbox Live Token...");
                    self.xbl_response = self.msa_response.get_xbl_reponse(client).await;
                }
                info!("Refreshing Xsts Token...");
                self.xsts_response = self.xbl_response.xbl_to_xsts_response(client).await;
            }
            info!("Refreshing Minecraft Token...");
            self.mc_response = self.xsts_response.xsts_to_mc_response(client).await;

            debug!("Saving updated account details...");
            update_account(previous, self.clone());
        }
    }
}

