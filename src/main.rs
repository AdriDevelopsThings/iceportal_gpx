use std::{time::Duration, fs::File};

use gpx::{Gpx, Track, TrackSegment, Waypoint, write, GpxVersion};
use iceportal::{ICEPortal, status::GpsStatus};
use geo_types::Point;

fn close_and_write(g: Gpx) {
    let file = File::create("output.gpx")
        .expect("Error while creating file");
    write(&g, file)
        .expect("Error while writing gpx data to file");
}

#[tokio::main]
async fn main() {
    let mut g = Gpx { version: GpxVersion::Gpx11, ..Default::default() };
    let mut track = Track::new();
    let trip = ICEPortal::fetch_trip_info().await
        .expect("Error while sending request to iceportal api: check if you are connected to the bahn wifi");
    track.name = Some(trip.trip.train_type + trip.trip.vzn.as_str());
    g.tracks.push(track);
    let track = g.tracks.get_mut(0).unwrap();
    let track_segment = TrackSegment::new();
    track.segments.push(track_segment);
    let track_segment = track.segments.get_mut(0).unwrap();
    let (cancel_sender, mut cancel) = tokio::sync::oneshot::channel();
    tokio::spawn(async {
        tokio::signal::ctrl_c().await.unwrap();
        cancel_sender.send(()).unwrap();
    });

    let (status_sender, mut status_receiver) = tokio::sync::mpsc::unbounded_channel();

    tokio::spawn(async move {
        loop {
            let status = ICEPortal::fetch_status().await;
            if let Ok(train) = status { 
                if train.gps_status == GpsStatus::Valid {
                    let waypoint = Waypoint::new(Point::new(train.longitude, train.latitude));                    
                    status_sender.send(waypoint).expect("lol");
                    println!("Fetched location...");
                } else {
                    println!("GPSStatus is invalid");
                }
            } else if let Err(e) = status {
                println!("Error while fetching: {:?}, retrying in 5 seconds...", e);
            }
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    });
    loop {
        tokio::select! {
            Some(status) = status_receiver.recv() => {
                track_segment.points.push(status);
            },
            _ = &mut cancel => {
                println!("Pressed Ctrl-c");
                close_and_write(g);
                break;
            },
        }
    }
}
