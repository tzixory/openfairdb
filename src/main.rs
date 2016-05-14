// Copyright (c) 2015 - 2016 Markus Kohlhase <mail@markus-kohlhase.de>

#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]

#[macro_use]
extern crate log;
extern crate env_logger;
#[macro_use]
extern crate nickel;
extern crate rustc_serialize;
extern crate hyper;
extern crate unicase;
extern crate clap;
#[macro_use]
extern crate rusted_cypher;
extern crate r2d2;
extern crate r2d2_cypher;
extern crate typemap;
extern crate uuid;
extern crate geojson;
extern crate nickel_cookies as cookies;
extern crate nickel_session as session;
extern crate time;

mod json;
mod store;
mod error;
mod search;
mod filter;
mod sort;
mod geo;

use nickel::{Nickel, JsonBody, MediaType, QueryString, Request, Response, MiddlewareResult, HttpRouter};
use nickel::status::StatusCode;
use hyper::header::{AccessControlAllowOrigin, AccessControlAllowHeaders, AccessControlAllowMethods};
use hyper::method::Method;
use r2d2_cypher::CypherConnectionManager;
use clap::{Arg,App};
use json::{Entry, Category, SearchResult};
use store::Store;
use error::{AppError, ParameterError, StoreError, AccessError};
use rustc_serialize::json::encode;
use filter::{FilterByCategoryIds, FilterByBoundingBox};
use search::Search;
use sort::SortByDistanceTo;
use session::{Session, CookieSession};
use cookies::SecretKey;
use time::Duration;

static VERSION                  : &'static str = "0.0.16";
static POOL_SIZE                : u32 = 5;
static MAX_INVISIBLE_RESULTS    : usize = 5;
static SECRET_KEY               : &'static SecretKey = &SecretKey([0; 32]);

#[derive(Debug, Clone)]
struct Data {
  db: r2d2::Pool<CypherConnectionManager>
}

impl cookies::KeyProvider for Data {
  fn key(&self) -> SecretKey { SECRET_KEY.clone() }
}

impl session::Store for Data {
  type Session = Option<String>;

  fn timeout() -> Duration {
    Duration::seconds(5)
  }
}

fn enable_cors<'mw>(_req: &mut Request<Data>, mut res: Response<'mw,Data>) -> MiddlewareResult<'mw,Data> {
  res.set(AccessControlAllowOrigin::Any);
  res.set(AccessControlAllowHeaders(vec![
      "Origin".into(),
      "X-Requested-With".into(),
      "Content-Type".into(),
      "Accept".into(),
  ]));

  res.next_middleware()
}

fn extract_ids(s: &str) -> Vec<String> {
  s.split(",")
   .map(|x|x.to_owned())
   .filter(|id| id != "")
   .collect::<Vec<String>>()
}

#[test]
fn extract_ids_test(){
  assert_eq!(extract_ids("abc"), vec!("abc"));
  assert_eq!(extract_ids("a,b,c"), vec!("a","b","c"));
  assert_eq!(extract_ids("").len(), 0);
  assert_eq!(extract_ids("abc,,d"), vec!("abc","d"));
}

fn main() {

  env_logger::init().unwrap();

  let matches = App::new("openFairDB")
    .version(VERSION)
    .author("Markus Kohlhase <mail@markus-kohlhase.de>")
    .arg(Arg::with_name("port")
      .short("p")
      .long("port")
      .value_name("PORT")
      .default_value("6767")
      .help("Set the port to listen"))
    .arg(Arg::with_name("db-url")
      .long("db-url")
      .value_name("URL")
      .default_value("http://neo4j:neo4j@127.0.0.1:7474/db/data")
      .help("URL to the Neo4j database"))
    .arg(Arg::with_name("enable-cors")
      .long("enable-cors")
      .help("Allow requests from any origin"))
    .get_matches();

  let db_url  = matches.value_of("db-url").unwrap().to_owned();
  let manager = CypherConnectionManager{url:db_url};
  let config  = r2d2::Config::builder().pool_size(POOL_SIZE).build();
  let pool    = r2d2::Pool::new(config, manager).unwrap();
  let data    = Data{db: pool};

  let mut server = Nickel::with_data(data);

  if matches.is_present("enable-cors") {
    server.utilize(enable_cors);
    server.options("/entries/*", middleware!{|_, mut res|
      res.set(AccessControlAllowHeaders(vec!["Content-Type".into()]));
      res.set(AccessControlAllowMethods(vec![Method::Get, Method::Post, Method::Put]));
      StatusCode::Ok
    });
  }

  server.utilize(router! {

    get "/entries/:id" => |req, mut res|{
      match req.param("id")
        .ok_or(ParameterError::Id).map_err(AppError::Parameter)
        .and_then(|s|{
          let ids = extract_ids(s);
          let data: &Data = res.server_data();
          data.db.clone().get()
            .map_err(StoreError::Pool)
            .map_err(AppError::Store)
            .and_then(|ref pool|{

              Entry::all(pool).map_err(AppError::Store).and_then(|entries|

                match ids.len() {
                  0 => encode(&entries).map_err(AppError::Encode),
                  1 => entries
                      .iter()
                      .find(|x| x.id == Some(ids[0].clone()))
                      .ok_or(StoreError::NotFound).map_err(AppError::Store)
                      .and_then(|e| encode(&e).map_err(AppError::Encode)),
                  _ => encode(&entries
                        .iter()
                        .filter(|e| e.id.is_some())
                        .filter(|e| ids.iter().any(|id| e.id == Some(id.clone())))
                        .collect::<Vec<&Entry>>()).map_err(AppError::Encode)
                }
              )
            })
        })
        {
          Ok(x)  => {
            res.set(MediaType::Json);
            (StatusCode::Ok, format!("{}",x))
          },
          Err(ref err) =>
            (err.into(), format!("Could not fetch entries: {}", err))
        }
      }

    post "/entries/:id" => |req, res|{
      match req.json_as::<Entry>().map_err(AppError::Io)
        .and_then(|json|{
          let data: &Data = res.server_data();
          data.db.clone().get()
            .map_err(StoreError::Pool)
            .map_err(AppError::Store)
            .and_then(|ref pool|
              json.save(pool)
                .map_err(AppError::Store)
                .and_then(|e| e.id
                   .ok_or(StoreError::Save).map_err(AppError::Store))

          )
      })
      {
        Ok(id)  => (StatusCode::Ok, format!("{}",id)),
        Err(ref err) =>
          (err.into(), format!("Could not create entry: {}", err))
      }
    }

    put "/entries/:id" => |req, mut res|{
      let entry = req.json_as::<Entry>();
      let data: &Data = res.server_data();
      match req.param("id")
        .ok_or(ParameterError::Id).map_err(AppError::Parameter)
        .and_then(|id| entry.map_err(AppError::Io)
          .and_then(|mut new_data|{
            data.db.clone().get()
            .map_err(StoreError::Pool)
            .map_err(AppError::Store)
            .and_then(|ref pool|
              Entry::get(pool, id.to_owned())
              .map_err(AppError::Store)
              .and_then(|_|{
                new_data.id = Some(id.to_owned());
                new_data.save(pool)
                .map_err(AppError::Store)
                .and_then(|_| Ok(id))
              })
            )
          })
        )
      {
        Ok(id) => {
          (StatusCode::Ok, format!("{}",id))
        }
        Err(ref err) => {
          let msg = format!("Could not save entry: {}", err);
          warn!("{}", msg);
          (err.into(), format!("Could not save entry: {}", err))
        }
      }
    }

    get "/categories/:id" => |req, mut res|{
      match req.param("id")
        .ok_or(ParameterError::Id).map_err(AppError::Parameter)
        .and_then(|s|{
          let ids = extract_ids(s);
          let data: &Data = res.server_data();
          data.db.clone().get()
            .map_err(StoreError::Pool)
            .map_err(AppError::Store)
            .and_then(|ref pool|{
              Category::all(pool)
                .map_err(AppError::Store)
                .and_then(|categories|
                  match ids.len() {
                    0 => encode(&categories).map_err(AppError::Encode),
                    1 => {
                        let id = Some(ids[0].clone());
                        categories
                          .iter()
                          .find(|x| x.id == id)
                          .ok_or(StoreError::NotFound).map_err(AppError::Store)
                          .and_then(|e| encode(&e).map_err(AppError::Encode))
                    },
                    _ =>
                      encode(&categories.iter()
                       .filter(|e| e.id.is_some())
                       .filter(|e| ids.iter().any(|id| e.id == Some(id.clone())))
                       .collect::<Vec<&Category>>()).map_err(AppError::Encode)
                  }
              )
          })
        })
        {
          Ok(x)  => {
            res.set(MediaType::Json);
            (StatusCode::Ok, format!("{}",x))
          },
          Err(ref err) =>
            (err.into(), format!("Could not fetch categories: {}", err))
        }
      }

    get "/search" => |req, mut res| {
      let data: &Data = res.server_data();
      let query = req.query();
      match query
        .get("bbox")
        .ok_or(ParameterError::Bbox).map_err(AppError::Parameter)
        .and_then(|bbox_str| query
        .get("categories")
        .ok_or(ParameterError::Categories).map_err(AppError::Parameter)
        .and_then(|cat_str| data.db.clone().get()
            .map_err(StoreError::Pool)
            .map_err(AppError::Store)
            .and_then(|ref pool|
              Entry::all(pool)
              .map_err(AppError::Store)
              .and_then(|entries|{

                let cat_ids = extract_ids(cat_str);
                let bbox = try!(geo::extract_bbox(bbox_str));
                let bbox_center = geo::center(&bbox[0], &bbox[1]);
                let cat_filtered_entries = &entries
                  .filter_by_category_ids(&cat_ids);

                let mut pre_filtered_entries = match query.get("text"){
                  Some(txt) => cat_filtered_entries.filter_by_search_text(&txt.to_owned()),
                  None      => cat_filtered_entries.iter().map(|x|x.clone()).collect()
                };

                pre_filtered_entries.sort_by_distance_to(&bbox_center);

                let visible_results = pre_filtered_entries
                  .filter_by_bounding_box(&bbox)
                  .map_to_ids();

                let invisible_results = pre_filtered_entries
                  .iter()
                  .filter(|e| !visible_results.iter().any(|v| e.id == Some(v.clone()) ))
                  .take(MAX_INVISIBLE_RESULTS)
                  .map(|x|x.clone())
                  .collect::<Vec<_>>()
                  .map_to_ids();

                let search_result = SearchResult{
                  visible   : visible_results,
                  invisible : invisible_results
                };
                encode(&search_result).map_err(AppError::Encode)
              })
            )
      ))
      {
        Ok(x)  => {
          res.set(MediaType::Json);
          (StatusCode::Ok, format!("{}", x))
        },
        Err(ref err) =>
          (err.into(), format!("Could not search entries: {}", err))
      }

    }

    post "/user/login" => |req, mut res| {
      let data: &Data = res.server_data();
      match req.json_as::<json::User>()
        .map_err(AppError::Io)
        .and_then(|u|
          data.db.clone().get()
          .map_err(StoreError::Pool)
          .map_err(AppError::Store)
          .and_then(|ref pool|
            json::User::get(pool, u.name.clone())
            .map_err(AppError::Store)
            .and_then(|user|{
              let n = user.name;
              if u.name == n && u.password == user.password {
                *CookieSession::get_mut(req, &mut res) = Some(n);
                Ok(())
              } else {
                Err(AccessError::Denied).map_err(AppError::Access)
              }
          }
        )))
      {
        Ok(_)        => (StatusCode::Ok, format!("Successfully logged in")),
        Err(ref err) => (err.into(), format!("Could not login: {}", err))
      }
    }

    get "/server/version" => { VERSION }

  });

  server.listen((
    "127.0.0.1",
    matches.value_of("port").unwrap().parse::<u16>().unwrap()));

}
