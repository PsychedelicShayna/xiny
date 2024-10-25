use std::fs::{self, File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::PathBuf;

use anyhow::{self as ah, Context};
use dirs;

#[derive(Debug, Clone)]
pub struct ConfigFile {
    pub values: Config,
    pub path: PathBuf,
}

impl ConfigFile {
    pub fn new() -> ah::Result<Self> {
        let mut config_file = ConfigFile {
            values: Config::default(),
            path: PathBuf::new(),
        };

        let config_dir = dirs::config_dir()
            .ok_or(ah::anyhow!("Could not find config directory."))
            .context("Config struct finding config directory via dirs::config_dir()")?;

        let config_file_path = config_dir.join("xiny.conf");

        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)
                .context("Config struct creating config directory via fs::create_dir_all()")?;
        }

        if !config_file_path.exists() {
            let mut file = File::create(&config_file_path)
                .context("Config struct creating config file via File::create()")?;

            let default_config = Config::default();
            let config_str = default_config.dump();

            file.write_all(config_str.as_bytes())
                .context("Config struct writing default config to file via Write::write_all()")?;

            config_file.values = default_config;
            config_file.path = config_file_path;
        } else {
            let file = OpenOptions::new()
                .read(true)
                .open(&config_file_path)
                .context(
                    "Config struct opening config file via OpenOptions::new().read(true).open()",
                )?;

            let mut reader = BufReader::new(file);
            let mut config_str = String::new();

            reader
                .read_to_string(&mut config_str)
                .context("Config struct reading config file via BufReader::read_to_string()")?;

            config_file.values = Config::parse(&config_str);
            config_file.path = config_file_path;
        }

        Ok(config_file)
    }

    pub fn write_changes(&self) -> ah::Result<()> {
        let file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&self.path)
            .context("ConfigFile write_changes OpenOptions opening file")?;

        let mut writer = BufWriter::new(file);
        let config_str = self.values.dump();

        writer
            .write_all(config_str.as_bytes())
            .context("ConfigFile write_changes writing to file")?;

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    pub repo: String,
    pub branch: String,
    pub langs: Vec<String>,
    pub renderer: String,
    pub first: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            repo: "https://github.com/adambard/learnxinyminutes-docs.git".into(),
            branch: "master".into(),
            langs: vec![],
            renderer: "glow".into(),
            first: true,
        }
    }
}

impl Config {
    pub fn update(&mut self, config: &str) {
        *self = Config::parse(config);
    }

    pub fn is_valid_key(key: &str) -> bool {
        matches!(key, "repo" | "branch" | "langs" | "renderer" | "first")
    }

    pub fn set_value(&mut self, key: &str, value: &str) -> ah::Result<()> {
        match key {
            "repo" => self.repo = value.into(),
            "branch" => self.branch = value.into(),
            "langs" => self.langs = value.split(',').map(|s| s.into()).collect(),
            "renderer" => self.renderer = value.into(),
            "first" if matches!(key, "true" | "false") => self.first = value.parse().unwrap(),
            _ => ah::bail!("Invalid config assignment {} = {}", key, value),
        };

        Ok(())
    }

    pub fn get_value(&self, key: &str) -> Option<String> {
        match key {
            "repo" => Some(self.repo.clone()),
            "branch" => Some(self.branch.clone()),
            "langs" => Some(self.langs.join(",")),
            "renderer" => Some(self.renderer.clone()),
            "first" => Some(self.first.to_string()),
            _ => None,
        }
    }

    pub fn parse(config: &str) -> Self {
        let mut template = Config::default();
        let lines = config.lines();

        for line in lines {
            let parts: Vec<&str> = line.splitn(2, ':').collect();

            if parts.len() != 2 {
                eprintln!("Invalid config line: {}", line);
                std::process::exit(1);
            }

            let key = parts[0].trim();
            let value = parts[1].trim();

            match key {
                "repo" => template.repo = value.into(),
                "branch" => template.branch = value.into(),
                "langs" => template.langs = value.split(',').map(|s| s.into()).collect(),
                "renderer" => template.renderer = value.into(),
                "first" => template.first = value.parse().unwrap(),
                _ => eprintln!("Unknown config key: {}", key),
            }
        }

        template
    }

    pub fn dump(&self) -> String {
        let mut config = String::new();

        config.push_str(&format!("repo: {}\n", self.repo));
        config.push_str(&format!("branch: {}\n", self.branch));
        config.push_str(&format!("langs: {}\n", self.langs.join(",")));
        config.push_str(&format!("renderer: {}\n", self.renderer));
        config.push_str(&format!("first: {}\n", self.first));

        config
    }
}
