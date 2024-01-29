use std::{io::Error, fs::{self, create_dir_all}, path::PathBuf};

use log::*;

use crate::launcher::authentication::auth_structs::*;

use super::{consts::ACCOUNT_FILE_NAME, utils::get_config_dir};


fn get_accounts_path() -> PathBuf {
    let accounts_path = get_config_dir().join(ACCOUNT_FILE_NAME);
    if let Some(parent) = accounts_path.parent() {
        if !parent.exists() {
            create_dir_all(parent).expect("Failed to create config directory!")
        }
    }
    accounts_path
}

fn read_accounts_safe() -> AccountList {
    let accounts_path = get_accounts_path();

    if let Ok(file) = fs::read_to_string(&accounts_path) {
        if let Ok(account_list) = serde_json::from_str(&file) {
            return account_list
        }
    }
    let fallback_list = AccountList {
        accounts: Vec::new(),
        selected_index: None,
    };

    fs::write(accounts_path, serde_json::to_string_pretty(&fallback_list).unwrap()).expect("Failed to write to accounts file");
    fallback_list
}

pub fn load_accounts() -> Vec<MCAccount> {
    read_accounts_safe().accounts
}

pub fn get_accounts() -> Vec<MCProfile> {
    load_accounts().into_iter().map(|acc| {
        acc.mc_profile
    }).collect()
}

pub fn get_active_account() -> Option<MCAccount> {
    let accounts = load_accounts();
    if let Some(index) = get_selected_index() {
        let i: usize = index.try_into().unwrap_or(0);
        accounts.into_iter().nth(i)
    } else { None }
}

pub fn save_accounts(accounts: AccountList) {
    fs::write(
        get_accounts_path(), 
        serde_json::to_string_pretty(&accounts).expect("Failed to serialize accounts to json")
    ).expect("Failed to write to accounts.json");
}

pub fn save_new_account(account: MCAccount) -> Result<(), Error> {
    let mut acc_list = read_accounts_safe();
    acc_list.accounts.push(account);
    set_selected_index((acc_list.accounts.len()-1).try_into().unwrap_or(0));
    save_accounts(acc_list);
    Ok(())
}

pub fn update_account(account: MCAccount, new_data: MCAccount) {
    let mut acc_list = read_accounts_safe();
    debug!("Pre: {:#?}", acc_list);

    for acc in acc_list.accounts.iter_mut() {
        if *acc == account {
            *acc = new_data;
            break;
        }
    }

    debug!("Post: {:#?}", acc_list);
    save_accounts(acc_list)
}

pub fn remove_account(index: usize) {
    let mut acc_list = read_accounts_safe();

    acc_list.accounts.remove(index);
    save_accounts(acc_list);
}

pub fn get_selected_index() -> Option<u32> {
    read_accounts_safe().selected_index
}

pub fn set_selected_index(index: u32) {
    let mut accounts = read_accounts_safe();
    accounts.selected_index = Some(index);
    save_accounts(accounts);
}