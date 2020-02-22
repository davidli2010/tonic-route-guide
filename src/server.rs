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

//! Route guide server.

// reference: https://github.com/hyperium/tonic/blob/master/examples/src/routeguide/server.rs

use futures::{Stream, StreamExt};
use std::pin::Pin;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::mpsc;
use tonic::{Request, Response, Status, Streaming};

use route::route_guide_server::{RouteGuide, RouteGuideServer};
use route::{util, Feature, FeatureDatabase, Point, Rectangle, RouteNote, RouteSummary};
use std::collections::HashMap;
use tonic::transport::Server;

#[derive(Debug)]
struct RouteGuideService {
    features: Arc<FeatureDatabase>,
}

#[tonic::async_trait]
impl RouteGuide for RouteGuideService {
    async fn get_feature(&self, request: Request<Point>) -> Result<Response<Feature>, Status> {
        match self
            .features
            .feature
            .iter()
            .find(|&f| f.location.as_ref() == Some(request.get_ref()))
        {
            Some(f) => Ok(Response::new(f.clone())),
            None => Ok(Response::new(Feature::default())),
        }
    }

    type ListFeaturesStream = mpsc::Receiver<Result<Feature, Status>>;

    async fn list_features(
        &self,
        request: Request<Rectangle>,
    ) -> Result<Response<Self::ListFeaturesStream>, Status> {
        let (mut tx, rx) = mpsc::channel(4);
        let features = self.features.clone();

        tokio::spawn(async move {
            for f in features.feature.iter() {
                if util::in_range(f.location.as_ref().unwrap(), request.get_ref()) {
                    tx.send(Ok(f.clone())).await.unwrap();
                }
            }
        });

        Ok(Response::new(rx))
    }

    async fn record_route(
        &self,
        request: Request<Streaming<Point>>,
    ) -> Result<Response<RouteSummary>, Status> {
        let mut stream = request.into_inner();

        let mut summary = RouteSummary::default();
        let mut prev_point = None;
        let timer = Instant::now();

        while let Some(point) = stream.next().await {
            let point = point?;
            summary.point_count += 1;

            self.features.feature.iter().for_each(|f| {
                if f.location.as_ref() == Some(&point) {
                    summary.feature_count += 1;
                }
            });

            if let Some(ref prev) = prev_point {
                summary.distance += util::calc_distance(prev, &point);
            }

            prev_point = Some(point);
        }

        summary.elapsed_time = timer.elapsed().as_secs() as i32;

        Ok(Response::new(summary))
    }

    type RouteChatStream =
        Pin<Box<dyn Stream<Item = Result<RouteNote, Status>> + Send + Sync + 'static>>;

    async fn route_chat(
        &self,
        request: Request<Streaming<RouteNote>>,
    ) -> Result<Response<Self::RouteChatStream>, Status> {
        let mut notes = HashMap::new();
        let mut stream = request.into_inner();

        let output = async_stream::try_stream! {
          while let Some(note) = stream.next().await {
            let note = note?;

            let location = note.location.clone().unwrap();

            let location_notes = notes.entry(location).or_insert(vec![]);
            location_notes.push(note);

            for note in location_notes {
                yield note.clone();
            }
          }
        };

        Ok(Response::new(Box::pin(output)))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:8980".parse().unwrap();

    let route_guide = RouteGuideService {
        features: Arc::new(util::load_database()),
    };

    let service = RouteGuideServer::new(route_guide);

    println!("listening on {}", addr);

    Server::builder().add_service(service).serve(addr).await?;

    Ok(())
}
