extern crate hidapi;

use clap::{arg, command};
use co2mon_rs::{decrypt, is_encrypted, parse_record, validate, Config, Record, get_topic_suffix};
use color_eyre::eyre::Result;
use hidapi::HidApi;
use rumqttc::{Client, MqttOptions, QoS};
use std::time::Duration;
use std::{env, thread};
use tracing::{debug, info, warn};
use tracing_subscriber::fmt::format;

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let cfg: Config = confy::load("co2mon-rs", None)?;
    let matches = command!()
        .arg(arg!(--mqtt_broker <URL>).required(true))
        .arg(arg!(--mqtt_topic <TOPIC_NAME>).default_value("#"))
        .get_matches();
    let mqtt_broker = matches
        .get_one::<String>("mqtt_broker")
        .expect("Has default.");
    let mqtt_topic = matches
        .get_one::<String>("mqtt_topic")
        .expect("Has default.");
    let mqtt_user = env::var("MQTT_USER")?;
    let mqtt_pass = env::var("MQTT_PASS")?;

    let mut mqttoptions = MqttOptions::new("rumqtt-sync", mqtt_broker, 1883);
    mqttoptions.set_credentials(mqtt_user, mqtt_pass);
    mqttoptions.set_keep_alive(Duration::from_secs(5));

    let (mut client, mut connection) = Client::new(mqttoptions, 10);
    let key: [u8; 8] = rand::random();
    // const HIDIOCSFEATURE_9: u32 = 0xC0094806;
    let api = HidApi::new()?;
    let mut report: Vec<u8> = vec![0x00];
    report.extend_from_slice(&key);
    info!("Sending feature report: {:?}", report);
    let device = api.open(cfg.vid, cfg.pid)?; // find actual hid device
    device.send_feature_report(&report)?;

    let mut buf: [u8; 8] = Default::default();

    thread::spawn(move || {
        connection
            .iter()
            .for_each(|notification| debug!("Notification = {:?}", notification));
    });

    loop {
        device.read(&mut buf)?;
        if is_encrypted(&buf) {
            decrypt(&key, &mut buf);
        }

        if let Err(e) = validate(&buf) {
            warn!("Validation failed for: {:?} - {:?}", &buf, e);
            continue;
        }

        let record = Record {
            key: buf[0],
            value: (buf[1] as u16) << 8 | buf[2] as u16,
        };

        if let Some(metric) = parse_record(record) {
            info!("{:?}", metric);
            client
                .publish(
                    format!("{}/{}", mqtt_topic, get_topic_suffix(&metric)),
                    QoS::AtLeastOnce,
                    false,
                    serde_json::to_string(&metric)?,
                )
                .unwrap();
        }
    }

    Ok(())
}
