use self::commands::docker::{
    create_docker_container, delete_docker_container, get_containers, start_docker_container,
    stop_docker_container,
};

use app_state::AppState;

use std::env;
use tauri::{Builder, Manager};
use tauri_plugin_sql::{Migration, MigrationKind};

mod app_state;
mod bc;
mod bc_container;
mod commands;
mod git;
mod utils;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let migrations = vec![Migration {
        version: 1,
        description: "create_initial_tables",
        sql:
            "CREATE TABLE containers (id INTEGER PRIMARY KEY, name TEXT, image TEXT, status TEXT);",
        kind: MigrationKind::Up,
    }];
    tauri::Builder::default()
        .setup(|app| {
            app.manage(AppState::default());
            Ok(())
        })
        .plugin(
            tauri_plugin_sql::Builder::default()
                .add_migrations("sqlite:mydatabase.db", migrations)
                .build(),
        )
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            create_docker_container,
            delete_docker_container,
            get_containers,
            start_docker_container,
            stop_docker_container
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
