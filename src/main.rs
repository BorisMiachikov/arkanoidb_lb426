mod app;
mod components;
mod events;
mod plugins;
mod resources;
mod setup;
mod systems;

fn main() {
    app::build_app().run();
}
