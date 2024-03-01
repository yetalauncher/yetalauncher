use std::{io::Error, fs::{self, create_dir_all}, path::PathBuf};

use log::*;
use slint::{ModelRc, VecModel};

use crate::{launcher::authentication::auth_structs::*, slint_generatedMainWindow::{SlMCAccount, SlMCSkin}};

use super::{consts::ACCOUNT_FILE_NAME, slint_utils::SlintOption, utils::get_config_dir};



impl Accounts {
    pub fn get() -> Accounts {
        let accounts_path = Self::get_path();
    
        if let Ok(file) = fs::read_to_string(&accounts_path) {
            if let Ok(account_list) = serde_json::from_str(&file) {
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

    pub fn get_active_account() -> Option<MCAccount> {
        let acc_list = Self::get();

        if let Some(index) = acc_list.selected_index {
            let i: usize = index.try_into().unwrap_or(0);
            acc_list.accounts.into_iter().nth(i)
        } else { None }
    }

    pub fn save(&self) {
        debug!("Saving accounts...");
        fs::write(
            Self::get_path(), 
            serde_json::to_string_pretty(self).expect("Failed to serialize accounts to json")
        ).expect("Failed to write to accounts.json");
    }

    pub fn save_new_account(account: MCAccount) -> Result<(), Error> {
        let mut acc_list = Accounts::get();
        acc_list.accounts.push(account);
        acc_list.selected_index = Some((acc_list.accounts.len() - 1) as u32);
        acc_list.save();
        Ok(())
    }

    pub fn update_account(account: MCAccount, new_data: MCAccount) {
        let mut acc_list = Accounts::get();

        for acc in acc_list.accounts.iter_mut() {
            if *acc == account {
                *acc = new_data;
                break;
            }
        }

        acc_list.save();
    }

    pub fn remove_account(index: usize) {
        let mut acc_list = Accounts::get();

        acc_list.accounts.remove(index);
        acc_list.save();
    }
}

impl MCAccount {
    pub fn to_slint(&self) -> SlMCAccount {
        SlMCAccount {
            username: self.mc_profile.name.to_string().into(),
            uuid: self.mc_profile.id.to_string().into(),
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