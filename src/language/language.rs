use super::iana_tags as iana;
use anyhow as ah;

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct Language {
    pub region: String,
    pub language: String,
    pub region_tag: String,
    pub language_tag: String,
    pub tag: String,
}

impl Language {
    pub fn from_tag(tag: &str) -> ah::Result<Self> {
        let parts: Vec<&str> = tag.splitn(2, '-').collect();

        if parts.len() != 2 {
            ah::bail!(
                "Invalid language tag format; expected 'language-region', but have '{:?}' parts.",
                parts
            );
        }

        let language_tag: String = parts[0].trim().to_lowercase();
        let region_tag: String = parts[1].trim().to_lowercase();

        let language = lang_tag_to_desc(&language_tag)
            .ok_or(ah::anyhow!("Unknown language tag {}", language_tag))?;

        // In case the tag wasn't found as a region, try looking it up as a
        // language (written to solve no-nb)
        let region = region_tag_to_desc(&region_tag)
            .or_else(|| lang_tag_to_desc(&region_tag))
            .ok_or(ah::anyhow!("Unknown region tag {}", &region_tag))?;

        Ok(Self {
            language: language.to_string(),
            region: region.to_string(),
            language_tag,
            region_tag,
            tag: tag.to_string(),
        })
    }

    pub fn to_tag(&self) -> String {
        format!("{}-{}", self.language_tag, self.region_tag)
    }
}

pub fn lang_tag_to_desc(tag: &str) -> Option<&str> {
    iana::LANG_TAG_TO_DESC.get(tag).map(|v| &**v)
}

pub fn region_tag_to_desc(tag: &str) -> Option<&str> {
    iana::REGION_TAG_TO_DESC.get(tag).map(|v| &**v)
}

pub fn lang_desc_to_tag(tag: &str) -> Option<&str> {
    iana::LANG_DESC_TO_TAG.get(tag).map(|v| &**v)
}

pub fn region_des_to_tag(tag: &str) -> Option<&str> {
    iana::REGION_DESC_TO_TAG.get(tag).map(|v| &**v)
}
