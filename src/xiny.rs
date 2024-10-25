use std::{
    collections::{BTreeMap as Map, HashSet},
    ffi::OsStr,
    fs::DirEntry,
    io,
    path::PathBuf,
};

use crate::lang::Language;
use anyhow::{self as ah, Context};

pub type SubjectName = String;

#[derive(Debug, Clone, Default)]
pub struct SubjectFiles {
    pub files: Map<Language, PathBuf>,
}

impl SubjectFiles {
    pub fn available_languages(&self) -> Vec<&Language> {
        self.files.keys().collect()
    }

    pub fn get_in_language(&self, lang: &Language) -> Option<&PathBuf> {
        self.files.get(lang)
    }

    pub fn add_translation(&mut self, lang: Language, path: PathBuf) {
        self.files.insert(lang, path);
    }
}

#[derive(Debug, Clone, Default)]
pub struct XinY {
    /// The root directory of the repository. The .git directory should be
    /// contained within this directory.
    pub root_dir: PathBuf,

    /// Maps every subject name to a list of its corresponding Markdown files
    /// in every available language.
    pub subjects: Map<SubjectName, SubjectFiles>,
}

// Don't rely on the language in the filenames
// some have them some don'. Rely on the folder
// to determine the language.
//
// html-ar.html.markdown
// bash.html.markdown

impl XinY {
    pub fn get_subject(&self, subject: &str) -> Option<&SubjectFiles> {
        self.subjects.get(subject)
    }

    pub fn get_subject_in(&self, subject: &str, lang: &Language) -> Option<&PathBuf> {
        self.subjects
            .get(subject)
            .and_then(|sf| sf.get_in_language(lang))
    }

    pub fn get_available_languages(&self) -> HashSet<&Language> {
        self.subjects
            .values()
            .flat_map(SubjectFiles::available_languages)
            .collect()
    }

    /// Collects and stores the subjects found in the given directory in the
    /// internal subjects map. If a language is provided, it will be used in
    /// place of the directory name language tag (mainly for the root). This
    /// will not do so recurisvley, it only adds from the top level.
    pub fn collect_subjects(&mut self, path: &PathBuf, language: Language) -> ah::Result<()> {
        if !path.is_dir() {
            ah::bail!("XinY::collect_subjects path is not a directory");
        }

        let read_dir = path
            .read_dir()
            .context("XinY::collect_subjects reading directory")?;

        for entry in read_dir.filter_map(Result::ok) {
            let path = entry.path();

            if !path.is_file() {
                continue;
            }

            let name: &str = match path.file_name().map(OsStr::to_str).flatten() {
                Some(name) => name.trim(),
                None => {
                    eprintln!("Skipping path {:?}; invalid UTF-8 in name.", path);
                    continue;
                }
            };

            if !name.ends_with(".html.markdown") {
                continue;
            }

            // Get rid of the language or region tag from the filename.
            let filter_out = format!("-{}.html", &language.language_tag);
            let name = name.replace(&filter_out, ".html");
            let filter_out = format!("-{}.html", &language.region_tag);
            let name = name.replace(&filter_out, ".html");

            let subject_name: &str = name
                .splitn(2, '.')
                .next()
                .context("XinY::collect_subjects splitting entry name by '.'")?;

            self.subjects
                .entry(subject_name.to_string())
                .or_default()
                .add_translation(language.clone(), path);
        }

        Ok(())
    }

    pub fn available_subjects(&self) -> Vec<&SubjectName> {
        self.subjects.keys().collect()
    }

    pub fn subject_available_in(&self, subject: &SubjectName) -> Vec<&Language> {
        self.subjects
            .get(subject)
            .map(SubjectFiles::available_languages)
            .unwrap_or_default()
    }

    /// This will identify every language directory in the root directory and
    /// call `collect_subjects` on each of them, alongside the root directory
    /// itself, where the language is forced to English.
    pub fn collect_from_root(&mut self, root_dir: &PathBuf) -> ah::Result<()> {
        let read_dir = root_dir
            .read_dir()
            .context("XinY::collect_from_root reading root directory")?;

        for entry in read_dir.filter_map(Result::ok) {
            let path = entry.path();

            let metadata = match entry.metadata() {
                Ok(m) => m,
                Err(e) => {
                    eprintln!(
                        "Error reading metadata for entry: {:?} @ {:?}; skipping.",
                        e, path
                    );
                    continue;
                }
            };

            if !metadata.is_dir() {
                continue;
            }

            let name: &str = match path.file_name().map(OsStr::to_str).flatten() {
                Some(name) => name,
                None => {
                    eprintln!("Skipping path {:?}; invalid UTF-8 in name.", path);
                    continue;
                }
            };

            let language = match Language::from_tag(name) {
                Ok(lang) => lang,
                Err(_) => {
                    eprintln!("Skipping path {:?}; invalid language tag.", path);
                    continue;
                }
            };

            self.collect_subjects(&path, language)
                .context("XinY::collect_from_root collecting subjects")?;
        }

        self.collect_subjects(
            root_dir,
            Language::from_tag("en-us")
                .context("XinY::collect_from_root forcing English as root language")?,
        )
        .context("XinY::collect_from_root collecting root subjects")?;

        Ok(())
    }

    pub fn new(root_dir: &PathBuf) -> ah::Result<Self> {
        let mut xiny = Self::default();

        xiny.collect_from_root(root_dir)
            .context("XinY::new collecting subjects")?;

        Ok(xiny)
    }
}
