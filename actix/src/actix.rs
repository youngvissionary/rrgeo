#[macro_use]
extern crate failure;
extern crate reverse_geocoder;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate actix_web;
extern crate queryst;
extern crate serde;
extern crate time;
#[macro_use]
extern crate serde_derive;
use actix_web::{error, http, middleware, web, App, HttpServer, HttpResponse, Result};

use reverse_geocoder::{
    Locations,
    Record,
    ReverseGeocoder,
};

use failure::Error;

#[derive(Fail, Debug)]
enum MyError {
    #[fail(display = "bad request")]
    BadClientData,
    #[fail(display = "not found")]
    NotFound,
}

impl error::ResponseError for MyError {
    fn error_response(&self) -> HttpResponse {
        match *self {
            MyError::BadClientData => HttpResponse::new(http::StatusCode::BAD_REQUEST),
            MyError::NotFound => HttpResponse::new(http::StatusCode::BAD_REQUEST),
        }
    }
}

lazy_static! {
    static ref LOCATIONS: Locations = Locations::from_memory();
    static ref GEOCODER: ReverseGeocoder<'static> = ReverseGeocoder::new(&LOCATIONS);
}

#[derive(Deserialize)]
struct LatLong {
    lat: f64,
    long: f64,
}

async fn index(lat_long: web::Query<LatLong>) -> Result<web::Json<Record>, Error> {
    let res = GEOCODER.search(&[lat_long.lat, lat_long.long])?;

    match res.len() {
        0 => Err(Error::from(MyError::NotFound)),
        _ => Ok(web::Json((*((res.get(0).unwrap()).1)).clone())),
    }
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            // .wrap(middleware::Logger::default())
            .route("/", web::get().to(index))
    })
    .keep_alive(10)
    .bind("127.0.0.1:3000")?
    .run()
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    extern crate bytes;
    use self::bytes::Bytes;

    use actix_web::dev::Service;
    use actix_web::{http, test, web, App};
    use super::index;

    #[actix_rt::test]
    async fn it_serves_results_on_actix() -> Result<(), Error> {
        let mut app = test::init_service(
            App::new().route("/", web::get().to(index))
        )
        .await;

        let req = test::TestRequest::get().uri("/?lat=44.962786&long=-93.344722").to_request();

        let resp = app.call(req).await.unwrap();

        assert_eq!(resp.status(), http::StatusCode::OK);

        let response_body = match resp.response().body().as_ref() {
            Some(actix_web::body::Body::Bytes(bytes)) => bytes,
            _ => panic!("Response error"),
        };

        assert_eq!(*response_body, Bytes::from_static(b"{\"lat\":44.9483,\"lon\":-93.34801,\"name\":\"Saint Louis Park\",\"admin1\":\"Minnesota\",\"admin2\":\"Hennepin County\",\"admin3\":\"US\"}"));

        Ok(())
    }
}
