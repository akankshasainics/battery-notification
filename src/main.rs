#[macro_use]
extern crate ref_thread_local;
use std::env;
use std::path::PathBuf;
use std::{fs, time::Duration};
extern crate notify_rust;
extern crate rodio;
use notify_rust::Notification;
use ref_thread_local::RefThreadLocal;
use rodio::{OutputStream, OutputStreamHandle};
use rodio::source::{SineWave, Source};

const DEFAULT_BATTERY_DIR: &'static str = "/sys/class/power_supply/BAT0/";
const STATUS_FILE: &'static str = "status";
const CAPACITY_FILE: &'static str = "capacity";
const FIRST_WARNING_BATTERY: u32 = 10;
const SECOND_WARNING_BATTERY: u32 = 5;

ref_thread_local! {
    static managed STREAM: (OutputStream, OutputStreamHandle) = OutputStream::try_default().unwrap();
}

fn is_dry_run() -> bool {
    env::var("DRY_RUN").unwrap_or("0".to_string()) == "1"
}

fn get_battery_dir() -> PathBuf {
    let path_str= env::var("BATTERY_DIR").unwrap_or(DEFAULT_BATTERY_DIR.to_string());
    path_str.into()
}

fn is_charging() -> bool {
    let filename = get_battery_dir().join(STATUS_FILE);
    let contents = fs::read_to_string(filename)
        .expect("Something went wrong reading the file");
    if contents.trim().eq("Discharging"){
        false
    } else {
        true
    }
}

fn battery_capacity() -> u32 {
    let filename = get_battery_dir().join(CAPACITY_FILE);
    let contents = fs::read_to_string(filename)
        .expect("Something went wrong reading the file");
    let battery_status: i32 = contents.trim().parse().unwrap();
    battery_status as u32
}

fn play_sound(duration: Duration) {
    if is_dry_run() {
        eprintln!("Dry run: PlayAudio");
        return
    }
    eprintln!("Playing Audio");
    let source = SineWave::new(440).take_duration(duration).amplify(0.20);
    let result = STREAM.borrow().1.play_raw(source.convert_samples());
    if result.is_err(){
        println!("Can't play audio, {:?}", result);
    }
}

fn send_notification(battery_capacity: u32) {
    if is_dry_run() {
        eprintln!("Dry run: Send Notification {}", battery_capacity);
        return
    }
    let result = Notification::new()
    .summary("Please Connect Charger")
    .body(&("Remaining Battery power is ".to_string() + &battery_capacity.to_string() + " %, You might want to connect charger"))
    .icon("dialog-information")
    .show();
    if result.is_err(){
        println!("Can't send notifcation, {:?}", result);
    }
    
}

fn act(){
    let time_gap_bt_notifcations: u64 = 300;
    if is_charging(){
        return;
    }
    let battery_status = battery_capacity();
    if battery_status <= FIRST_WARNING_BATTERY {
        send_notification(battery_status);
    }
    while battery_capacity() <= SECOND_WARNING_BATTERY && !is_charging(){
        play_sound(Duration::from_secs_f32(0.25));
        std::thread::sleep(std::time::Duration::from_secs(time_gap_bt_notifcations));
    }
}

fn main() {
        let time_interval: u64 = 300;
        loop {
            act();
            std::thread::sleep(std::time::Duration::from_secs(time_interval));
        }
}