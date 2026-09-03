// command to allow for the automated scraping of require auth cookies
// LEETCODE_SESSION and csrftoken.

use pookie::enums::Cookie;

use crate::{
    config::Config,
    error::{LeetCodeError, Result},
};
use std::{
    io::{self, Write},
    process::Command,
};

const RESET: &str = "\x1B[0m";
const BOLD: &str = "\x1B[1m";
const CYAN: &str = "\x1B[1;36m";
const DIM: &str = "\x1B[2m";
const CLEAR_SCREEN: &str = "\x1B[2J\x1B[1;1H";

const LEETCODE_DOMAIN: &str = "leetcode.com";
const LEETCODE_LOGIN: &str = "https://leetcode.com/accounts/login/";

pub fn login() -> Result<()> {
    print_login_prompt();
    open_in_firefox(LEETCODE_LOGIN);
    await_enter();

    let (session, csrf) = extract_cookies_from_browser()?;
    save_credentials_to_config(&session, &csrf)?;

    Ok(())
}

fn print_login_prompt() {
    print!("{CLEAR_SCREEN}");
    println!("{CYAN}{BOLD}╔═══════════════════════════════════════╗{RESET}");
    println!("{CYAN}{BOLD}║              LC-CLI LOGIN             ║{RESET}");
    println!("{CYAN}{BOLD}╚═══════════════════════════════════════╝{RESET}");
    println!();
    println!("  1. Log into LeetCode in your browser, if you haven't already:");
    println!();
    println!("     {BOLD}{CYAN}{LEETCODE_LOGIN}{RESET}");
    println!();
    println!("  2. Once logged in, press Enter here: the required tokens will");
    println!("     be taken from your browser, and stored in the config file. ");

    println!();
    println!("{DIM}  (Nothing is sent anywhere - this reads cookies directly");
    println!("   from your local browser's storage.){RESET}");
    println!();
}

/// launches firefox without erroring if not installed
fn open_in_firefox(url: &str) {
    let _ = Command::new("open")
        .arg("-a")
        .arg("Firefox")
        .arg(url)
        .status();
}

// blocks the thread until the user presses enter to continue
fn await_enter() {
    print!("Press Enter once you have logged in...........");
    io::stdout().flush().unwrap();
    let mut buf = String::new();
    io::stdin().read_line(&mut buf).ok();
}

/// currently only scrapes the following browsers: chrome, firefox
fn extract_cookies_from_browser() -> Result<(String, String)> {
    let cookies = pookie::chrome(Some(vec![LEETCODE_DOMAIN.to_string()]))
        .or_else(|_| pookie::firefox(Some(vec![LEETCODE_DOMAIN.to_string()])))?;
    extract_session_and_csrf(&cookies)
}

fn extract_session_and_csrf(cookies: &[Cookie]) -> Result<(String, String)> {
    let session = cookies
        .iter()
        .find(|c| c.name == "LEETCODE_SESSION")
        .map(|c| c.value.to_string())
        .ok_or_else(|| LeetCodeError::NotLoggedIn)?;
    let csrf = cookies
        .iter()
        .find(|c| c.name == "csrftoken")
        .map(|c| c.value.to_string())
        .ok_or_else(|| LeetCodeError::NotLoggedIn)?;
    Ok((session, csrf))
}

fn save_credentials_to_config(session: &str, csrf: &str) -> Result<()> {
    let mut config = Config::load()?;
    config.leetcode_session = Some(session.to_string());
    config.csrf_token = Some(csrf.to_string());
    config.save()?;

    Ok(())
}
