use std::fs::{self, File};
use std::io::{BufReader, Read, Write};
use std::path::PathBuf;

use anyhow::{self as ah, Context};
use dirs;

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
            let file = File::open(&config_file_path)
                .context("Config struct opening config file via File::open()")?;

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

    fn write_changes(&self) -> ah::Result<()> {
        let mut file = File::open(&self.path).context("ConfigFile write_changes opening file")?;
        let config_str = self.values.dump();

        file.write_all(config_str.as_bytes())
            .context("ConfigFile write_changes writing to file")?;

        Ok(())
    }
}

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
