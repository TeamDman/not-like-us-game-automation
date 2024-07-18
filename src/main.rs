use image::Rgba;
use itertools::Itertools;
use winput::Button;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::Duration;
use winc::prelude::get_all_monitors;
use winc::prelude::get_monitor_capturer;
use winc::prelude::FromCorners;
use winc::prelude::HasLeft;
use winc::prelude::Metrics;
use winc::prelude::Translatable;
use winc::prelude::RECT;
use winput::message_loop;
use winput::message_loop::Event;
use winput::Action;

fn main() {
    // declare the owl locations (right wing edge ish)
    let bottom_owl = (-812, 687);
    let right_owl = (-669, 523);
    let left_owl = (-950, 519);
    let top_owl = (-809, 366);
    let owl_positions = [bottom_owl, right_owl, left_owl, top_owl];

    // find the monitor
    let monitor = get_all_monitors()
        .unwrap()
        .into_iter()
        .min_by_key(|monitor| monitor.info.rect.left())
        .expect("getting left monitor failed");
    let monitor = Rc::new(monitor);
    // println!("Monitor: {:?}", monitor.info.name);

    let padding = 30;
    let capturers = owl_positions
        .iter()
        .map(|coord| {
            let capture_region = RECT::from_corners(
                coord.translate(-padding, -padding),
                coord.translate(padding, padding),
            );
            // dbg!(capture_region);
            get_monitor_capturer(monitor.clone(), capture_region)
        })
        .collect_vec();
    // dbg!(monitor.info.rect);

    let image_dir = PathBuf::from_iter(["target", "capture"]);
    std::fs::create_dir_all(&image_dir).expect("getting image dir failed");

    let target_rgb = (117, 113, 120);
    let fuzz = 30;
    let get_active_owl = || {
        for (i, capturer) in capturers.iter().enumerate() {
            let img = capturer
                .capture(&mut Metrics::None)
                .expect("getting capture failed");
            img.save(image_dir.join(format!("owl-{}.png", i)))
                .expect("saving image failed");
            let Rgba([r, g, b, _]) = img.get_pixel(padding as u32, padding as u32);
            let r_dist = (*r as i32 - target_rgb.0).abs();
            let g_dist = (*g as i32 - target_rgb.1).abs();
            let b_dist = (*b as i32 - target_rgb.2).abs();
            if (r_dist < fuzz) && (g_dist < fuzz) && (b_dist < fuzz) {
                return Some(owl_positions[i]);
            } else {
                println!(
                    "Owl {} is not active: {:?} {:?} {:?}",
                    i, r_dist, g_dist, b_dist
                );
            }
        }
        None
    };

    let receiver = message_loop::start().expect("failed to start message loop");

    'outer: loop {
        while let Some(event) = receiver.try_next_event() {
            if let Event::Keyboard {
                action: Action::Press,
                vk: winput::Vk::Oem3,
                ..
            } = event
            {
                println!("Exiting");
                break 'outer;
            }
        }
        if let Some(owl) = get_active_owl() {
            println!("Active owl: {:?}", owl);
            winput::Mouse::set_position(owl.0 - 15, owl.1 + 15)
                .expect("failed to set mouse position");
            std::thread::sleep(Duration::from_millis(rand::random::<u64>() % 50));
            winput::send(Button::Left);
            std::thread::sleep(Duration::from_millis(rand::random::<u64>() % 300));
        } else {
            std::thread::sleep(Duration::from_millis(rand::random::<u64>() % 200));
            // break;
        }
    }
}
