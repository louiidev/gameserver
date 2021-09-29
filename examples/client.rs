use gameserver::{parse_request, serialize_request, Client, InputState, Message};

use std::{collections::HashMap, mem::size_of_val, net::SocketAddr, sync::Arc};

use bincode::serialize;
use smol_rs::{
    math::{Vector, Vector2},
    App, AppSettings, Color, Keycode, Rectangle,
};
use tokio::{net::UdpSocket, sync::mpsc};

#[tokio::main]
pub async fn main() {
    let mut client = Client::new();

    let mut app = App::new(AppSettings {
        target_fps: 300.,
        ..Default::default()
    });

    let mut position = Vector::from([0., 0.]);

    while app.is_running() {
        let keys = app.input.get_pressed_keys();
        let mut direction = Vector2::default();
        for key in keys.iter() {
            match key {
                Keycode::Up => direction.y -= 1.,
                Keycode::Down => direction.y += 1.,
                Keycode::Right => direction.x += 1.,
                Keycode::Left => direction.x -= 1.,
                _ => {}
            }
        }

        if !keys.is_empty() {
            client.send_input(InputState {
                direction: (direction.x, direction.y),
                delta: app.delta,
            });
            let snapshot = client.poll();
            if let Some(snapshot) = snapshot {
                println!("snapshot: {:?}", snapshot);
                // println!("position: {:?}", &_pos);

                position.x = snapshot.player_position.0;
                position.y = snapshot.player_position.1;
            }
        }

        // position = _pos;

        app.renderer.clear(Color::BLACK);

        app.renderer.rectangle(
            Rectangle {
                x: position.x,
                y: position.y,
                width: 50.,
                height: 50.,
            },
            Color::WHITE,
        );

        app.end_scene();
    }

    // let r = Arc::new(socket);
    // let (tx, mut rx) = mpsc::channel::<(Vec<u8>, SocketAddr)>(1_000);
}
