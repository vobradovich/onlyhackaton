use onlyhack_client::GetProfilesResponse;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    Model,
    Fan,
}

impl Role {
    pub fn toggle(self) -> Self {
        match self {
            Self::Model => Self::Fan,
            Self::Fan => Self::Model,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Model => "MODEL",
            Self::Fan => "FAN",
        }
    }
}

#[derive(Clone, Debug)]
pub struct FanHiddenRow {
    pub content_id: u64,
    pub preview: String,
    pub price: u128,
}

#[derive(Clone, Debug)]
pub struct PurchaseRecord {
    pub buyer: u64,
    pub content_id: u64,
    pub price: u128,
}

#[derive(Clone, Debug)]
pub struct ModelPaidRow {
    pub content_id: u64,
    pub preview: String,
    pub price: u128,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PromptStep {
    Name,
    About,
    Preview,
    Plaintext,
    Price,
}

#[derive(Clone, Debug)]
pub struct CreateProfilePrompt {
    pub step: PromptStep,
    pub name: String,
    pub about: String,
    pub input: String,
}

impl CreateProfilePrompt {
    pub fn new() -> Self {
        Self {
            step: PromptStep::Name,
            name: String::new(),
            about: String::new(),
            input: String::new(),
        }
    }

    pub fn title(&self) -> &'static str {
        match self.step {
            PromptStep::Name => "Create Profile: Name",
            PromptStep::About => "Create Profile: About",
            _ => "Create Profile",
        }
    }

    pub fn submit_current_step(&mut self) -> Option<(String, String)> {
        match self.step {
            PromptStep::Name => {
                self.name = self.input.trim().to_string();
                self.input.clear();
                self.step = PromptStep::About;
                None
            }
            PromptStep::About => {
                self.about = self.input.trim().to_string();
                Some((self.name.clone(), self.about.clone()))
            }
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct AddPaidPrompt {
    pub step: PromptStep,
    pub preview: String,
    pub plaintext: String,
    pub price_input: String,
    pub input: String,
}

impl AddPaidPrompt {
    pub fn new() -> Self {
        Self {
            step: PromptStep::Preview,
            preview: String::new(),
            plaintext: String::new(),
            price_input: String::new(),
            input: String::new(),
        }
    }

    pub fn title(&self) -> &'static str {
        match self.step {
            PromptStep::Preview => "Add Paid Content: Preview",
            PromptStep::Plaintext => "Add Paid Content: Plaintext",
            PromptStep::Price => "Add Paid Content: Price",
            _ => "Add Paid Content",
        }
    }

    pub fn submit_current_step(&mut self) -> Option<(String, Vec<u8>, u128)> {
        match self.step {
            PromptStep::Preview => {
                self.preview = self.input.trim().to_string();
                self.input.clear();
                self.step = PromptStep::Plaintext;
                None
            }
            PromptStep::Plaintext => {
                self.plaintext = self.input.clone();
                self.input.clear();
                self.step = PromptStep::Price;
                None
            }
            PromptStep::Price => {
                self.price_input = self.input.trim().to_string();
                let price = self.price_input.parse::<u128>().ok()?;
                Some((self.preview.clone(), self.plaintext.clone().into_bytes(), price))
            }
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct AppState {
    pub role: Role,
    pub profiles: GetProfilesResponse,
    pub fan_hidden: Vec<FanHiddenRow>,
    pub model_paid: Vec<ModelPaidRow>,
    pub selected_hidden: usize,
    pub model_balance: u128,
    pub fan_balance: u128,
    pub history: Vec<PurchaseRecord>,
    pub decrypted: String,
    pub status: String,
    pub model_profile_created: bool,
    pub create_profile_prompt: Option<CreateProfilePrompt>,
    pub add_prompt: Option<AddPaidPrompt>,
    pub key_path: String,
}

impl AppState {
    pub fn new(key_path: String) -> Self {
        Self {
            role: Role::Fan,
            profiles: GetProfilesResponse { profiles: Vec::new() },
            fan_hidden: Vec::new(),
            model_paid: Vec::new(),
            selected_hidden: 0,
            model_balance: 0,
            fan_balance: 0,
            history: Vec::new(),
            decrypted: String::new(),
            status: "Ready".to_string(),
            model_profile_created: false,
            create_profile_prompt: None,
            add_prompt: None,
            key_path,
        }
    }

    pub fn selected_content_id(&self) -> Option<u64> {
        self.fan_hidden
            .get(self.selected_hidden)
            .map(|row| row.content_id)
    }

    pub fn move_selection_down(&mut self) {
        if self.fan_hidden.is_empty() {
            self.selected_hidden = 0;
            return;
        }
        self.selected_hidden = (self.selected_hidden + 1) % self.fan_hidden.len();
    }

    pub fn move_selection_up(&mut self) {
        if self.fan_hidden.is_empty() {
            self.selected_hidden = 0;
            return;
        }
        self.selected_hidden = if self.selected_hidden == 0 {
            self.fan_hidden.len() - 1
        } else {
            self.selected_hidden - 1
        };
    }

    pub fn normalize_selection(&mut self) {
        if self.fan_hidden.is_empty() {
            self.selected_hidden = 0;
            return;
        }
        if self.selected_hidden >= self.fan_hidden.len() {
            self.selected_hidden = self.fan_hidden.len() - 1;
        }
    }
}
