// PBHealth — entrypoint
//
// Em Windows, esconde o console em release.
// Em dev, mantém o console para logs.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    pbhealth_lib::run();
}