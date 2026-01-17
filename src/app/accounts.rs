use std::{fs::{self, create_dir_all}, path::PathBuf, sync::Arc};

use log::*;
use reqwest::Client;
use slint::{ModelRc, VecModel};

use crate::{launcher::authentication::auth_structs::*, slint_generatedMainWindow::{SlAccounts, SlMCAccount, SlMCSkin}, YetaLauncher};

use super::{consts::ACCOUNT_FILE_NAME, slint_utils::SlintOption, utils::get_config_dir};



impl Accounts {
    pub fn to_slint(&self) -> SlAccounts {
        SlAccounts {
            selected_index: SlintOption::from(self.selected_index.map(|i| i as i32)).into(),
            accounts: ModelRc::new(VecModel::from(
                self.accounts.iter()
                .enumerate()
                .map(|(i, acc)| acc.to_slint(i as i32))
                .collect::<Vec<_>>()
            ))
        }
    }

    pub fn get() -> Accounts {
        info!("Reading accounts...");
        let accounts_path = Self::get_path();
    
        if let Ok(file) = fs::read_to_string(&accounts_path) {
            if let Ok(account_list) = serde_json::from_str::<Accounts>(&file) {
                debug!("Successfully loaded {} account(s)", account_list.accounts.len());

                return account_list
            }
        }

        let fallback_list = Accounts {
            accounts: Vec::new(),
            selected_index: None,
        };
    
        fs::write(accounts_path, serde_json::to_string_pretty(&fallback_list).unwrap()).expect("Failed to write to accounts file");
        fallback_list
    }

    pub fn save(&self) {
        debug!("Saving accounts...");
        fs::write(
            Self::get_path(), 
            serde_json::to_string_pretty(self).expect("Failed to serialize accounts to json")
        ).expect("Failed to write to accounts.json");
    }

    fn get_path() -> PathBuf {
        let accounts_path = get_config_dir().join(ACCOUNT_FILE_NAME);
        if let Some(parent) = accounts_path.parent() {
            if !parent.exists() {
                info!("accounts.json file does not exist. Creating...");
                create_dir_all(parent).expect("Failed to create config directory!");
            }
        }
        accounts_path
    }

    pub async fn get_account_from_app(app: Arc<YetaLauncher>, client: &Client, force: bool) -> Option<MCAccount> {
        let mut accounts = {
            app.accounts.write().unwrap().clone()
        };

        accounts.get_active_account(client, force).await.cloned()
    }

    pub async fn get_active_account(&mut self, client: &Client, force: bool) -> Option<&MCAccount> {
        if let Some(index) = self.selected_index {
            let i: usize = index.try_into().unwrap_or(0);
            {
                let account = self.accounts.get_mut(i);
                if let Some(acc) = account {
                    acc.refresh(client, force).await;
                }
            }
            self.save();
            self.accounts.get(i)
        } else { None }
    }

    pub fn save_new_account(&mut self, account: MCAccount) {
        let existing = self.accounts.iter_mut()
        .enumerate()
        .find(
            |(_, acc)| acc.mc_profile.id == account.mc_profile.id
        );

        if let Some((i, acc)) = existing {
            *acc = account;
            self.selected_index = Some(i as u32);
            self.save();
        } else {
            self.accounts.push(account);
            self.selected_index = Some((self.accounts.len() - 1) as u32);
            self.save();
        }
    }

    pub fn update_account(&mut self, account: MCAccount, new_data: MCAccount) {
        for acc in self.accounts.iter_mut() {
            if *acc == account {
                *acc = new_data;
                break;
            }
        }
        self.save();
    }

    pub fn remove_account(&mut self, index: usize) {
        self.accounts.remove(index);
        self.save();
    }

    pub fn set_selected_index(&mut self, index: u32) {
        self.selected_index = Some(index);
        self.save()
    }
}

impl MCAccount {
    pub fn to_slint(&self, index: i32) -> SlMCAccount {
        SlMCAccount {
            username: self.mc_profile.name.to_string().into(),
            uuid: self.mc_profile.id.to_string().into(),
            index,
            capes: ModelRc::new(VecModel::from(
                self.mc_profile.capes.iter().map(
                    |cape| (cape.id.to_string().into(), )
                ).collect::<Vec<_>>()
            )),
            skins: ModelRc::new(VecModel::from(
                self.mc_profile.skins.iter().map(
                    MCSkin::to_slint
                ).collect::<Vec<_>>()
            ))
        }
    }
}

impl MCSkin {
    pub fn to_slint(&self) -> SlMCSkin {
        SlMCSkin {
            url: self.url.to_string().into(),
            state: self.state.to_string().into(),
            variant: self.variant.to_string().into(),
            alias: SlintOption::from(self.alias.clone()).into()
        }
    }
}