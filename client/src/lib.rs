#![no_std]

// Incorporate code generated based on the IDL file
include!("onlyhack_client.rs");

use sails_rs::prelude::*;

pub type GetProfileResponseTuple = (
    String,
    String,
    Vec<(u64, String)>,
    Vec<(u64, (String, String, u128))>,
);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OpenContentItem {
    pub content_id: u64,
    pub content: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PaidContentItem {
    pub content_id: u64,
    pub preview: String,
    pub data: String,
    pub price: u128,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PurchasedContentItem {
    pub content_id: u64,
    pub preview: String,
    pub price: u128,
    pub enc_content: Vec<u8>,
    pub m_under_pk: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HiddenContentItem {
    pub content_id: u64,
    pub preview: String,
    pub price: u128,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GetProfileResponse {
    pub name: String,
    pub about: String,
    pub open_content: Vec<OpenContentItem>,
    pub paid_content: Vec<PaidContentItem>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProfileSummary {
    pub name: String,
    pub about: String,
    pub open_content: Vec<OpenContentItem>,
    pub purchased_content: Vec<PurchasedContentItem>,
    pub hidden_content: Vec<HiddenContentItem>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GetProfilesResponse {
    pub profiles: Vec<ProfileSummary>,
}

impl From<(u64, String)> for OpenContentItem {
    fn from(value: (u64, String)) -> Self {
        let (content_id, content) = value;
        Self {
            content_id,
            content,
        }
    }
}

impl From<(u64, (String, String, u128))> for PaidContentItem {
    fn from(value: (u64, (String, String, u128))) -> Self {
        let (content_id, (preview, data, price)) = value;
        Self {
            content_id,
            preview,
            data,
            price,
        }
    }
}

impl From<(u64, String, u128, Vec<u8>, Vec<u8>)> for PurchasedContentItem {
    fn from(value: (u64, String, u128, Vec<u8>, Vec<u8>)) -> Self {
        let (content_id, preview, price, enc_content, m_under_pk) = value;
        Self {
            content_id,
            preview,
            price,
            enc_content,
            m_under_pk,
        }
    }
}

impl From<(u64, String, u128)> for HiddenContentItem {
    fn from(value: (u64, String, u128)) -> Self {
        let (content_id, preview, price) = value;
        Self {
            content_id,
            preview,
            price,
        }
    }
}

impl From<GetProfileResponseTuple> for GetProfileResponse {
    fn from(value: GetProfileResponseTuple) -> Self {
        let (name, about, open_content, paid_content) = value;
        Self {
            name,
            about,
            open_content: open_content.into_iter().map(Into::into).collect(),
            paid_content: paid_content.into_iter().map(Into::into).collect(),
        }
    }
}

impl
    From<(
        String,
        String,
        Vec<(u64, String)>,
        Vec<(u64, String, u128, Vec<u8>, Vec<u8>)>,
        Vec<(u64, String, u128)>,
    )> for ProfileSummary
{
    fn from(
        value: (
            String,
            String,
            Vec<(u64, String)>,
            Vec<(u64, String, u128, Vec<u8>, Vec<u8>)>,
            Vec<(u64, String, u128)>,
        ),
    ) -> Self {
        let (name, about, open_content, purchased_content, hidden_content) = value;
        Self {
            name,
            about,
            open_content: open_content.into_iter().map(Into::into).collect(),
            purchased_content: purchased_content.into_iter().map(Into::into).collect(),
            hidden_content: hidden_content.into_iter().map(Into::into).collect(),
        }
    }
}


pub fn format_get_profiles_markdown(response: &GetProfilesResponse) -> String {
    use core::fmt::Write;

    const RST: &str = "\x1b[0m";
    const BOLD: &str = "\x1b[1m";
    const CYAN: &str = "\x1b[36m";
    const GREEN: &str = "\x1b[32m";
    const YELLOW: &str = "\x1b[33m";
    const MAGENTA: &str = "\x1b[35m";
    const RED: &str = "\x1b[31m";

    let mut out = String::with_capacity(512 + response.profiles.len() * 512);
    out.push_str(CYAN);
    out.push_str(BOLD);
    out.push_str("+============================================================+\n");
    out.push_str("| [*]                GET PROFILES RESPONSE                   |\n");
    out.push_str("+============================================================+\n");
    out.push_str(RST);

    for (idx, profile) in response.profiles.iter().enumerate() {
        let _ = writeln!(
            out,
            "\n{}{BOLD}+------------------------ [@] PROFILE {:>3} ------------------------+{RST}",
            GREEN,
            idx + 1
        );
        let _ = writeln!(out, "{}| [N] Name  : {}{}", CYAN, profile.name, RST);
        let _ = writeln!(out, "{}| [I] About : {}{}", CYAN, profile.about, RST);
        out.push_str("|------------------------------------------------------------|\n");
        let _ = writeln!(out, "{}| [O] Open Content                                               {}", YELLOW, RST);
        for item in &profile.open_content {
            let _ = writeln!(
                out,
                "{}|  [+] [{}] {}{}",
                YELLOW, item.content_id, item.content, RST
            );
        }

        out.push_str("|------------------------------------------------------------|\n");
        let _ = writeln!(out, "{}| [$] Purchased Content                                          {}", MAGENTA, RST);
        for item in &profile.purchased_content {
            let _ = writeln!(
                out,
                "{}|  [$] [{}] {} | price {} | enc {}b | grant {}b{}",
                MAGENTA,
                item.content_id,
                item.preview,
                item.price,
                item.enc_content.len(),
                item.m_under_pk.len(),
                RST
            );
        }

        out.push_str("|------------------------------------------------------------|\n");
        let _ = writeln!(out, "{}| [X] Hidden Content                                             {}", RED, RST);
        for item in &profile.hidden_content {
            let _ = writeln!(
                out,
                "{}|  [x] [{}] {} | price {}{}",
                RED, item.content_id, item.preview, item.price, RST
            );
        }
        let _ = writeln!(out, "{}+------------------------------------------------------------+{}", GREEN, RST);
    }

    out
}
