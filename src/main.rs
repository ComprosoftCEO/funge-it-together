use global_state::GlobalState;
use level::load_all_level_packs;
use state::TitleState;

mod global_state;
mod grid;
mod isa;
mod level;
mod printable;
mod state;
mod statistics;

fn main() {
  let all_level_packs = match load_all_level_packs() {
    Ok(l) => l,
    Err(e) => {
      println!("Failed to load levels: {e}");
      return;
    },
  };

  let mut global_state = GlobalState::load(all_level_packs);

  state::run(Box::new(TitleState::new()), &mut global_state).ok();

  match global_state.save() {
    Ok(_) => println!("Saved progress!"),
    Err(e) => println!("Failed to save progress: {e}"),
  }
}
