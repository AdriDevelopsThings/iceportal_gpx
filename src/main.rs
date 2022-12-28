use std::{thread::sleep, time::Duration, fs::File};

use gpx::{Gpx, Track, TrackSegment, Waypoint, write};
use iceportal::ICEPortal;
use geo_types::Point;

fn main() {
    let mut g = Gpx::default();
    let mut track = Track::new();
    let trip = ICEPortal::fetch_trip_info().unwrap();
    track.name = Some(trip.trip.train_type + trip.trip.vzn.as_str());
    g.tracks.push(track);
    let track = g.tracks.get_mut(0)
        .expect("Error while sending request to iceportal api: check if you are connected to the bahn wifi");
    let track_segment = TrackSegment::new();
    track.segments.push(track_segment);
    let track_segment = track.segments.get_mut(0).unwrap();


    loop {
        let status = ICEPortal::fetch_status();
        if let Ok(train) = status {
            let waypoint = Waypoint::new(Point::new(train.latitude, train.longitude));
            track_segment.points.push(waypoint);
            println!("Fetched location...")
        } else if let Err(e) = status {
            println!("Error while fetching: {}, retrying in 5 seconds...", e);
        }
        sleep(Duration::from_secs(5));
    }
}
