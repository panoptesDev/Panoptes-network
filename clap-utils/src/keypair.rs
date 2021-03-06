use crate::{
    input_parsers::pubkeys_sigs_of,
    offline::{SIGNER_ARG, SIGN_ONLY_ARG},
    ArgConstant,
};
use bip39::{Language, Mnemonic, Seed};
use clap::ArgMatches;
use rpassword::prompt_password_stderr;
use solana_remote_wallet::{
    remote_keypair::generate_remote_keypair,
    remote_wallet::{maybe_wallet_manager, RemoteWalletError, RemoteWalletManager},
};
use solana_sdk::{
    hash::Hash,
    pubkey::Pubkey,
    signature::{
        keypair_from_seed, keypair_from_seed_phrase_and_passphrase, read_keypair,
        read_keypair_file, Keypair, NullSigner, Presigner, Signature, Signer,
    },
};
use std::{
    error,
    io::{stdin, stdout, Write},
    process::exit,
    str::FromStr,
    sync::Arc,
};

pub struct SignOnly {
    pub blockhash: Hash,
    pub present_signers: Vec<(Pubkey, Signature)>,
    pub absent_signers: Vec<Pubkey>,
    pub bad_signers: Vec<Pubkey>,
}

impl SignOnly {
    pub fn has_all_signers(&self) -> bool {
        self.absent_signers.is_empty() && self.bad_signers.is_empty()
    }

    pub fn presigner_of(&self, pubkey: &Pubkey) -> Option<Presigner> {
        presigner_from_pubkey_sigs(pubkey, &self.present_signers)
    }
}
pub type CliSigners = Vec<Box<dyn Signer>>;
pub type SignerIndex = usize;
pub struct CliSignerInfo {
    pub signers: CliSigners,
}

impl CliSignerInfo {
    pub fn index_of(&self, pubkey: Option<Pubkey>) -> Option<usize> {
        if let Some(pubkey) = pubkey {
            self.signers
                .iter()
                .position(|signer| signer.pubkey() == pubkey)
        } else {
            Some(0)
        }
    }
    pub fn index_of_or_none(&self, pubkey: Option<Pubkey>) -> Option<usize> {
        if let Some(pubkey) = pubkey {
            self.signers
                .iter()
                .position(|signer| signer.pubkey() == pubkey)
        } else {
            None
        }
    }
}

pub struct DefaultSigner {
    pub arg_name: String,
    pub path: String,
}

impl DefaultSigner {
    pub fn generate_unique_signers(
        &self,
        bulk_signers: Vec<Option<Box<dyn Signer>>>,
        matches: &ArgMatches<'_>,
        wallet_manager: &mut Option<Arc<RemoteWalletManager>>,
    ) -> Result<CliSignerInfo, Box<dyn error::Error>> {
        let mut unique_signers = vec![];

        // Determine if the default signer is needed
        if bulk_signers.iter().any(|signer| signer.is_none()) {
            let default_signer = self.signer_from_path(matches, wallet_manager)?;
            unique_signers.push(default_signer);
        }

        for signer in bulk_signers.into_iter() {
            if let Some(signer) = signer {
                if !unique_signers.iter().any(|s| s == &signer) {
                    unique_signers.push(signer);
                }
            }
        }
        Ok(CliSignerInfo {
            signers: unique_signers,
        })
    }

    pub fn signer_from_path(
        &self,
        matches: &ArgMatches,
        wallet_manager: &mut Option<Arc<RemoteWalletManager>>,
    ) -> Result<Box<dyn Signer>, Box<dyn std::error::Error>> {
        signer_from_path(matches, &self.path, &self.arg_name, wallet_manager)
    }
}

pub enum KeypairUrl {
    Ask,
    Filepath(String),
    Usb(String),
    Stdin,
    Pubkey(Pubkey),
}

pub fn parse_keypair_path(path: &str) -> KeypairUrl {
    if path == "-" {
        KeypairUrl::Stdin
    } else if path == ASK_KEYWORD {
        KeypairUrl::Ask
    } else if path.starts_with("usb://") {
        KeypairUrl::Usb(path.to_string())
    } else if let Ok(pubkey) = Pubkey::from_str(path) {
        KeypairUrl::Pubkey(pubkey)
    } else {
        KeypairUrl::Filepath(path.to_string())
    }
}

pub fn presigner_from_pubkey_sigs(
    pubkey: &Pubkey,
    signers: &[(Pubkey, Signature)],
) -> Option<Presigner> {
    signers.iter().find_map(|(signer, sig)| {
        if *signer == *pubkey {
            Some(Presigner::new(signer, sig))
        } else {
            None
        }
    })
}

pub fn signer_from_path(
    matches: &ArgMatches,
    path: &str,
    keypair_name: &str,
    wallet_manager: &mut Option<Arc<RemoteWalletManager>>,
) -> Result<Box<dyn Signer>, Box<dyn error::Error>> {
    match parse_keypair_path(path) {
        KeypairUrl::Ask => {
            let skip_validation = matches.is_present(SKIP_SEED_PHRASE_VALIDATION_ARG.name);
            Ok(Box::new(keypair_from_seed_phrase(
                keypair_name,
                skip_validation,
                false,
            )?))
        }
        KeypairUrl::Filepath(path) => match read_keypair_file(&path) {
            Err(e) => Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("could not read keypair file \"{}\". Run \"panoptes-keygen new\" to create a keypair file: {}", path, e),
            )
            .into()),
            Ok(file) => Ok(Box::new(file)),
        },
        KeypairUrl::Stdin => {
            let mut stdin = std::io::stdin();
            Ok(Box::new(read_keypair(&mut stdin)?))
        }
        KeypairUrl::Usb(path) => {
            if wallet_manager.is_none() {
                *wallet_manager = maybe_wallet_manager()?;
            }
            if let Some(wallet_manager) = wallet_manager {
                Ok(Box::new(generate_remote_keypair(
                    path,
                    wallet_manager,
                    matches.is_present("confirm_key"),
                    keypair_name,
                )?))
            } else {
                Err(RemoteWalletError::NoDeviceFound.into())
            }
        }
        KeypairUrl::Pubkey(pubkey) => {
            let presigner = pubkeys_sigs_of(matches, SIGNER_ARG.name)
                .as_ref()
                .and_then(|presigners| presigner_from_pubkey_sigs(&pubkey, presigners));
            if let Some(presigner) = presigner {
                Ok(Box::new(presigner))
            } else if matches.is_present(SIGN_ONLY_ARG.name) {
                Ok(Box::new(NullSigner::new(&pubkey)))
            } else {
                Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("missing signature for supplied pubkey: {}", pubkey),
                )
                .into())
            }
        }
    }
}

pub fn pubkey_from_path(
    matches: &ArgMatches,
    path: &str,
    keypair_name: &str,
    wallet_manager: &mut Option<Arc<RemoteWalletManager>>,
) -> Result<Pubkey, Box<dyn error::Error>> {
    match parse_keypair_path(path) {
        KeypairUrl::Pubkey(pubkey) => Ok(pubkey),
        _ => Ok(signer_from_path(matches, path, keypair_name, wallet_manager)?.pubkey()),
    }
}

pub fn resolve_signer_from_path(
    matches: &ArgMatches,
    path: &str,
    keypair_name: &str,
    wallet_manager: &mut Option<Arc<RemoteWalletManager>>,
) -> Result<Option<String>, Box<dyn error::Error>> {
    match parse_keypair_path(path) {
        KeypairUrl::Ask => {
            let skip_validation = matches.is_present(SKIP_SEED_PHRASE_VALIDATION_ARG.name);
            // This method validates the seed phrase, but returns `None` because there is no path
            // on disk or to a device
            keypair_from_seed_phrase(keypair_name, skip_validation, false).map(|_| None)
        }
        KeypairUrl::Filepath(path) => match read_keypair_file(&path) {
            Err(e) => Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("could not read keypair file \"{}\". Run \"panoptes-keygen new\" to create a keypair file: {}", path, e),
            )
            .into()),
            Ok(_) => Ok(Some(path.to_string())),
        },
        KeypairUrl::Stdin => {
            let mut stdin = std::io::stdin();
            // This method validates the keypair from stdin, but returns `None` because there is no
            // path on disk or to a device
            read_keypair(&mut stdin).map(|_| None)
        }
        KeypairUrl::Usb(path) => {
            if wallet_manager.is_none() {
                *wallet_manager = maybe_wallet_manager()?;
            }
            if let Some(wallet_manager) = wallet_manager {
                let path = generate_remote_keypair(
                    path,
                    wallet_manager,
                    matches.is_present("confirm_key"),
                    keypair_name,
                )
                .map(|keypair| keypair.path)?;
                Ok(Some(path))
            } else {
                Err(RemoteWalletError::NoDeviceFound.into())
            }
        }
        _ => Ok(Some(path.to_string())),
    }
}

// Keyword used to indicate that the user should be asked for a keypair seed phrase
pub const ASK_KEYWORD: &str = "ASK";

pub const SKIP_SEED_PHRASE_VALIDATION_ARG: ArgConstant<'static> = ArgConstant {
    long: "skip-seed-phrase-validation",
    name: "skip_seed_phrase_validation",
    help: "Skip validation of seed phrases. Use this if your phrase does not use the BIP39 official English word list",
};

/// Prompts user for a passphrase and then asks for confirmirmation to check for mistakes
pub fn prompt_passphrase(prompt: &str) -> Result<String, Box<dyn error::Error>> {
    let passphrase = prompt_password_stderr(&prompt)?;
    if !passphrase.is_empty() {
        let confirmed = rpassword::prompt_password_stderr("Enter same passphrase again: ")?;
        if confirmed != passphrase {
            return Err("Passphrases did not match".into());
        }
    }
    Ok(passphrase)
}

/// Reads user input from stdin to retrieve a seed phrase and passphrase for keypair derivation
/// Optionally skips validation of seed phrase
/// Optionally confirms recovered public key
pub fn keypair_from_seed_phrase(
    keypair_name: &str,
    skip_validation: bool,
    confirm_pubkey: bool,
) -> Result<Keypair, Box<dyn error::Error>> {
    let seed_phrase = prompt_password_stderr(&format!("[{}] seed phrase: ", keypair_name))?;
    let seed_phrase = seed_phrase.trim();
    let passphrase_prompt = format!(
        "[{}] If this seed phrase has an associated passphrase, enter it now. Otherwise, press ENTER to continue: ",
        keypair_name,
    );

    let keypair = if skip_validation {
        let passphrase = prompt_passphrase(&passphrase_prompt)?;
        keypair_from_seed_phrase_and_passphrase(&seed_phrase, &passphrase)?
    } else {
        let sanitized = sanitize_seed_phrase(seed_phrase);
        let parse_language_fn = || {
            for language in &[
                Language::English,
                Language::ChineseSimplified,
                Language::ChineseTraditional,
                Language::Japanese,
                Language::Spanish,
                Language::Korean,
                Language::French,
                Language::Italian,
            ] {
                if let Ok(mnemonic) = Mnemonic::from_phrase(&sanitized, *language) {
                    return Ok(mnemonic);
                }
            }
            Err("Can't get mnemonic from seed phrases")
        };
        let mnemonic = parse_language_fn()?;
        let passphrase = prompt_passphrase(&passphrase_prompt)?;
        let seed = Seed::new(&mnemonic, &passphrase);
        keypair_from_seed(seed.as_bytes())?
    };

    if confirm_pubkey {
        let pubkey = keypair.pubkey();
        print!("Recovered pubkey `{:?}`. Continue? (y/n): ", pubkey);
        let _ignored = stdout().flush();
        let mut input = String::new();
        stdin().read_line(&mut input).expect("Unexpected input");
        if input.to_lowercase().trim() != "y" {
            println!("Exiting");
            exit(1);
        }
    }

    Ok(keypair)
}

fn sanitize_seed_phrase(seed_phrase: &str) -> String {
    seed_phrase
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_seed_phrase() {
        let seed_phrase = " Mary   had\ta\u{2009}little  \n\t lamb";
        assert_eq!(
            "Mary had a little lamb".to_owned(),
            sanitize_seed_phrase(seed_phrase)
        );
    }
}
