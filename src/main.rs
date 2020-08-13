use dbus_tokio::connection;
use futures::future;
use dbus::channel::MatchingReceiver;
use dbus::message::MatchRule;
use dbus_crossroads::Crossroads;

use std::collections::HashMap;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

const SPEC_VERSION: &'static str = "1.2";

const CAPABILITIES: &'static [&'static str] = &[
    "action-icons",
    "actions",
    "body",
    "body-hyperlinks",
    "body-images",
    "body-markup",
    "icon-multi",
    "icon-static",
    "persistence",
    "sound"
];

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (resource, connection) = connection::new_session_sync()?;

    tokio::spawn(async {
        let err = resource.await;
        panic!("Connection to D-Bus lost: {}", err);
    });

    connection.request_name("org.freedesktop.Notifications", true, true, true).await?;

    let mut crossroads = Crossroads::new();

    crossroads.set_async_support(Some((connection.clone(), Box::new(|x| { tokio::spawn(x); }))));

    let iface_token = crossroads.register("org.freedesktop.Notifications", |builder| {
        builder.method("GetCapabilities", (), ("capabilities",), |_, _, _: ()| {
            Ok((CAPABILITIES,))
        });

        builder.method("GetServerInformation", (), ("name", "vendor", "version", "spec_version"), |_, _, _: ()| {
            Ok((
                "notifications",
                "arnaudsh",
                VERSION,
                SPEC_VERSION
            ))
        });

        builder.method(
            "Notify",
            ("app_name", "replaces_id", "app_icon", "summary", "body", "actions", "hints", "expire_timeout"),
            ("id",),
            |_, _, (name, replaces_id, app_icon, summary, body, actions, hints, expire_timeout): (String, u32, String, String, String, Vec<String>, HashMap<String, String>, i32)| {
                println!("Received a notification!");
                println!("name = {}", name);
                println!("replaces_id = {}", replaces_id);
                println!("app_icon = {}", app_icon);
                println!("summary = {}", summary);
                println!("body = {}", body);
                println!("actions = {:?}", actions);
                println!("hints = {:?}", hints);
                println!("expire_timeout = {}", expire_timeout);
                println!();

                // Should be an unique identifier but we don't care.
                Ok((42u32,))
        });
    });

    crossroads.insert("/org/freedesktop/Notifications", &[iface_token], ());

    connection.start_receive(MatchRule::new_method_call(), Box::new(move |message, connection| {
        crossroads.handle_message(message, connection).unwrap();
        true
    }));

    future::pending::<()>().await;

    Ok(())
}
