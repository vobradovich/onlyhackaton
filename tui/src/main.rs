mod sim;
mod state;
mod ui;

use std::{
    fs,
    io,
    path::{Path, PathBuf},
    time::Duration,
};

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use dleq_secret::{PK, Scalar, gen_keypair};
use ratatui::{Terminal, prelude::CrosstermBackend};
use sails_rs::{Decode, Encode, hex};
use serde::{Deserialize, Serialize};
use sim::{FAN_ID, MODEL_ID, Sim};
use state::{AddPaidPrompt, AppState, CreateProfilePrompt, FanHiddenRow, ModelPaidRow, PurchaseRecord, Role};

#[derive(Debug, Serialize, Deserialize)]
struct FanKeyDisk {
    sk_hex: String,
    pk_hex: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let key_path = PathBuf::from("fan_key.json");
    let fan_keys = load_or_create_fan_keys(&key_path)?;

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    let mut sim = rt.block_on(Sim::new()).map_err(io::Error::other)?;

    let mut app = AppState::new(key_path.to_string_lossy().to_string());
    refresh_data(&rt, &mut sim, &mut app)?;

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let run_result = run_loop(&rt, &mut terminal, &mut sim, &mut app, &fan_keys);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    run_result
}

fn run_loop(
    rt: &tokio::runtime::Runtime,
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    sim: &mut Sim,
    app: &mut AppState,
    fan_keys: &sim::FanKeys,
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        terminal.draw(|f| ui::draw(f, app))?;

        if !event::poll(Duration::from_millis(100))? {
            continue;
        }

        let Event::Key(key) = event::read()? else {
            continue;
        };
        if key.kind != KeyEventKind::Press {
            continue;
        }

        if !app.decrypted.is_empty() {
            match key.code {
                KeyCode::Char('q') => return Ok(()),
                KeyCode::Char('x') => {
                    app.decrypted.clear();
                    app.status = "Closed decrypted content popup".to_string();
                }
                _ => {}
            }
            continue;
        }

        if app.role == Role::Model && app.create_profile_prompt.is_some() {
            match key.code {
                KeyCode::Esc => {
                    app.create_profile_prompt = None;
                    app.status = "Canceled create profile prompt".to_string();
                }
                KeyCode::Backspace => {
                    if let Some(prompt) = &mut app.create_profile_prompt {
                        prompt.input.pop();
                    }
                }
                KeyCode::Enter => {
                    let result = app
                        .create_profile_prompt
                        .as_mut()
                        .and_then(|prompt| prompt.submit_current_step());

                    if let Some((name, about)) = result {
                        rt.block_on(sim.create_model_profile(name, about))
                            .map_err(io::Error::other)?;
                        app.create_profile_prompt = None;
                        refresh_data(rt, sim, app)?;
                        app.status = "Created/updated model profile".to_string();
                    } else {
                        app.status = "Step saved. Continue input and press Enter.".to_string();
                    }
                }
                KeyCode::Char(c) => {
                    if let Some(prompt) = &mut app.create_profile_prompt {
                        prompt.input.push(c);
                    }
                }
                _ => {}
            }
            continue;
        }

        if app.role == Role::Model && app.add_prompt.is_some() {
            match key.code {
                KeyCode::Esc => {
                    app.add_prompt = None;
                    app.status = "Canceled add paid content prompt".to_string();
                }
                KeyCode::Backspace => {
                    if let Some(prompt) = &mut app.add_prompt {
                        prompt.input.pop();
                    }
                }
                KeyCode::Enter => {
                    let result = app
                        .add_prompt
                        .as_mut()
                        .and_then(|prompt| prompt.submit_current_step());

                    if let Some((preview, plaintext, price)) = result {
                        if price == 0 {
                            app.status = "Invalid price. Enter a positive integer.".to_string();
                            continue;
                        }
                        let cid = rt
                            .block_on(sim.add_paid_content(preview, plaintext, price))
                            .map_err(io::Error::other)?;
                        app.add_prompt = None;
                        refresh_data(rt, sim, app)?;
                        app.status = format!("Added paid content id={cid}");
                    } else if let Some(prompt) = &app.add_prompt {
                        if prompt.step == state::PromptStep::Price {
                            app.status = "Invalid price. Enter a positive integer.".to_string();
                        } else {
                            app.status = "Step saved. Continue input and press Enter.".to_string();
                        }
                    }
                }
                KeyCode::Char(c) => {
                    if let Some(prompt) = &mut app.add_prompt {
                        prompt.input.push(c);
                    }
                }
                _ => {}
            }
            continue;
        }

        match key.code {
            KeyCode::Char('q') => return Ok(()),
            KeyCode::Tab => {
                app.role = app.role.toggle();
                app.status = format!("Switched role to {}", app.role.as_str());
            }
            KeyCode::Char('r') => {
                refresh_data(rt, sim, app)?;
                app.status = "Refreshed profiles and balances".to_string();
            }
            KeyCode::Down | KeyCode::Char('j') if app.role == Role::Fan => app.move_selection_down(),
            KeyCode::Up | KeyCode::Char('k') if app.role == Role::Fan => app.move_selection_up(),
            KeyCode::Char('c') if app.role == Role::Model => {
                app.create_profile_prompt = Some(CreateProfilePrompt::new());
                app.status = "Prompt started. Enter name and press Enter.".to_string();
            }
            KeyCode::Char('a') if app.role == Role::Model => {
                if !app.model_profile_created {
                    app.status = "Create profile first (press 'c').".to_string();
                    continue;
                }
                app.add_prompt = Some(AddPaidPrompt::new());
                app.status = "Prompt started. Enter preview and press Enter.".to_string();
            }
            KeyCode::Char('b') if app.role == Role::Fan => {
                if let Some(content_id) = app.selected_content_id() {
                    let (price, dec) = rt
                        .block_on(sim.buy_as_fan(content_id, fan_keys))
                        .map_err(io::Error::other)?;
                    app.decrypted = dec;
                    app.history.push(PurchaseRecord {
                        buyer: FAN_ID,
                        content_id,
                        price,
                    });
                    refresh_data(rt, sim, app)?;
                    app.status = format!("Bought content id={content_id}");
                } else {
                    app.status = "No content selected".to_string();
                }
            }
            _ => {}
        }
    }
}

fn refresh_data(
    rt: &tokio::runtime::Runtime,
    sim: &mut Sim,
    app: &mut AppState,
) -> Result<(), Box<dyn std::error::Error>> {
    app.profiles = rt
        .block_on(sim.get_profiles_for(FAN_ID))
        .map_err(io::Error::other)?;
    let model_profiles = rt
        .block_on(sim.get_profiles_for(MODEL_ID))
        .map_err(io::Error::other)?;
    app.model_profile_created = !model_profiles.profiles.is_empty();

    app.fan_hidden.clear();
    for profile in &app.profiles.profiles {
        for item in &profile.hidden_content {
            app.fan_hidden.push(FanHiddenRow {
                content_id: item.content_id,
                preview: item.preview.clone(),
                price: item.price,
            });
        }
    }
    app.normalize_selection();

    app.model_paid.clear();
    for profile in &model_profiles.profiles {
        for item in &profile.hidden_content {
            app.model_paid.push(ModelPaidRow {
                content_id: item.content_id,
                preview: item.preview.clone(),
                price: item.price,
            });
        }
    }

    let (model_balance, fan_balance) = sim.balances();
    app.model_balance = model_balance;
    app.fan_balance = fan_balance;
    Ok(())
}

fn load_or_create_fan_keys(path: &Path) -> Result<sim::FanKeys, Box<dyn std::error::Error>> {
    if path.exists() {
        let data = fs::read_to_string(path)?;
        let disk: FanKeyDisk = serde_json::from_str(&data)?;

        let sk_bytes = hex::decode(disk.sk_hex)?;
        if sk_bytes.len() != 32 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "invalid sk length").into());
        }
        let mut sk_arr = [0u8; 32];
        sk_arr.copy_from_slice(&sk_bytes);
        let sk = Scalar::from_bytes_mod_order(sk_arr);

        let pk_bytes = hex::decode(disk.pk_hex)?;
        let pk = PK::decode(&mut pk_bytes.as_slice())?.0;

        return Ok(sim::FanKeys { sk, pk });
    }

    let keypair = gen_keypair();
    let disk = FanKeyDisk {
        sk_hex: hex::encode(keypair.sk.to_bytes()),
        pk_hex: hex::encode(PK(keypair.pk).encode()),
    };
    fs::write(path, serde_json::to_string_pretty(&disk)?)?;

    Ok(sim::FanKeys {
        sk: keypair.sk,
        pk: keypair.pk,
    })
}
