// Copyright 2020 David Li
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Route guide client.

use futures::{stream, Future};
use rand::seq::SliceRandom;
use route::route_guide_client::RouteGuideClient;
use route::{util, Point, Rectangle, RouteNote};
use tokio::runtime::Runtime;
use tonic::Request;

trait FutureExt: Future {
    fn block_on(self, runtime: &mut Runtime) -> Self::Output;
}

impl<T: Future> FutureExt for T {
    #[inline]
    fn block_on(self, runtime: &mut Runtime) -> Self::Output {
        runtime.block_on(self)
    }
}

struct Client {
    client: RouteGuideClient<tonic::transport::Channel>,
    runtime: Runtime,
}

impl Client {
    fn new<T: Into<String>>(addr: T) -> Self {
        let mut runtime = Runtime::new().unwrap();
        let client = RouteGuideClient::connect(addr.into())
            .block_on(&mut runtime)
            .expect("");

        Self { client, runtime }
    }

    fn get_feature(&mut self, point: Point) {
        let feature = self
            .client
            .get_feature(Request::new(point.clone()))
            .block_on(&mut self.runtime)
            .expect("Failed to get feature")
            .into_inner();

        if feature.location.is_none() {
            eprintln!("Server returns incomplete feature.");
            return;
        }

        if feature.name.is_empty() {
            println!("No feature found at {}", point);
            return;
        }

        println!("Found feature {} at {}", feature.name, point);
    }

    fn list_features(&mut self, rect: Rectangle) {
        println!(
            "Searching features between {} and {}",
            rect.lo.as_ref().unwrap(),
            rect.hi.as_ref().unwrap(),
        );

        let mut response = self
            .client
            .list_features(rect)
            .block_on(&mut self.runtime)
            .expect("Failed to list features")
            .into_inner();

        loop {
            match response.message().block_on(&mut self.runtime) {
                Ok(Some(feature)) => {
                    let location = feature.location.as_ref().unwrap();
                    println!("Found feature {} at {}", feature.name, location);
                }
                Ok(None) => break,
                Err(e) => panic!("Failed to list features: {:?}", e),
            }
        }
        println!("List features successfully!");
    }

    fn record_route(&mut self) {
        let db = util::load_database();
        let mut rng = rand::thread_rng();

        let points: Vec<_> = (0..10)
            .map(|_| {
                let feature = db.feature.choose(&mut rng).unwrap();
                let location = feature.location.clone().unwrap();
                println!("Visiting {}", location);
                location
            })
            .collect();

        let request = Request::new(stream::iter(points));

        let sum = self
            .client
            .record_route(request)
            .block_on(&mut self.runtime)
            .unwrap()
            .into_inner();

        println!("Finished trip, route summary:");
        println!("\tVisited {} points", sum.point_count);
        println!("\tPassed {} features", sum.feature_count);
        println!("\tTravelled {} meters", sum.distance);
        println!("\tTook {} seconds", sum.elapsed_time);
    }

    fn route_chat(&mut self) {
        let notes: Vec<_> = vec![
            ("First message", 0, 0),
            ("Second message", 0, 1),
            ("Third message", 1, 0),
            ("Fourth message", 0, 0),
        ]
        .iter()
        .map(|(msg, lat, lon)| {
            println!("Sending message {} at ({},{})", msg, lat, lon);
            RouteNote {
                location: Some(Point {
                    latitude: *lat,
                    longitude: *lon,
                }),
                message: msg.to_string(),
            }
        })
        .collect();

        let request = Request::new(stream::iter(notes));

        let mut response = self
            .client
            .route_chat(request)
            .block_on(&mut self.runtime)
            .expect("Failed to route chat")
            .into_inner();

        loop {
            match response.message().block_on(&mut self.runtime) {
                Ok(Some(note)) => {
                    let location = note.location.as_ref().unwrap();
                    println!("Got message {} at {}", note.message, location);
                }
                Ok(None) => break,
                Err(e) => panic!("Failed to route chat: {:?}", e),
            }
        }
    }
}

fn main() {
    let mut client = Client::new("http://127.0.0.1:8980");

    println!("Get Feature:");
    // Looking for a valid feature
    client.get_feature(Point {
        latitude: 409146138,
        longitude: -746188906,
    });
    // Feature missing.
    client.get_feature(Point {
        latitude: 0,
        longitude: 0,
    });

    println!();
    println!("List features:");
    client.list_features(Rectangle {
        lo: Some(Point {
            latitude: 400000000,
            longitude: -750000000,
        }),
        hi: Some(Point {
            latitude: 420000000,
            longitude: -730000000,
        }),
    });

    println!();
    println!("Record route:");
    client.record_route();

    println!();
    println!("Route chat:");
    client.route_chat();
}
