use std::error::Error;
use std::net::IpAddr;
use std::str;
use std::sync::Mutex;
use local_ip_address::local_ip;
use actix_web::{get, post, web, App, HttpServer, Responder, HttpResponse};
use futures::StreamExt;
use serde::{Serialize, Deserialize};

const PORT: u16 = 3030;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Peer {
    ip: IpAddr,
    hostname: String
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Node {
    is_master:  bool,
    cur_master: Option<IpAddr>,
    ip:         IpAddr,
    peers:      Vec<Peer>,
    hostname:   String,
    latest_sel: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SendRequest {
    ip:       String,
    hostname: String,
    #[serde(with = "serde_bytes")]
    data:     Vec<u8>
}

pub fn init_node(master: bool) -> Result<Node, Box<dyn Error>> {
    let addr = local_ip().expect("Could not resolve local ip");
    //TODO scan IP range with nmap. If master node found, don't init as master.
    Ok(Node {
        is_master:  master,
        cur_master: match master { true => Some(addr), false => None },
        ip:         addr,
        peers:      Vec::new(),
        hostname:   hostname::get()?.into_string().expect("Could not resolve hostname"),
        latest_sel: Vec::new(),
    })
}

impl Node {
    async fn find_new_master(&mut self) -> Result<IpAddr, Box<dyn Error>> {

        Ok(self.ip)
    }

    async fn get_latest(&mut self) -> Result<(), Box<dyn Error>> {
        let client = reqwest::Client::new();
        if self.cur_master != None {
            let res = client.post(self.cur_master.unwrap().to_string())
                .json(&self).send().await?;
            println!("{:#?}", res);
            let body = res.text().await?;

            self.latest_sel = body.as_bytes().to_vec();
        }
        Ok(())
    }
/*
    async fn update_peers(&mut self) -> Result<(), Box<dyn Error>> {
        if !self.cur_master {
            let res = self.client.post(self.cur_master.unwrap().to_string())
                .json(&self).send().await?;
            println!("{:#?}", res);
        }
        Ok(())
    }
*/
    async fn send_sel(&mut self, data: Vec<u8>) -> Result<(), Box<dyn Error>> {

        Ok(())
    }
}

#[post("/latest")]
async fn latest(mut payload: web::Payload, data: web::Data<Mutex<Node>>)
    -> Result<HttpResponse, Box<dyn Error>> {
    let mut body: web::BytesMut = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        body.extend_from_slice(&chunk);
    }
    let peer = serde_json::from_slice::<Peer>(&body)?;
    println!("{:#?}", peer);

    let mut node = data.lock().unwrap();
    if !node.peers.contains(&peer) {
        node.peers.push(peer);
    }
    Ok(HttpResponse::Ok().json(str::from_utf8(&node.latest_sel.clone())?))
}

#[post("/peers")]
async fn peers(mut payload: web::Payload, data: web::Data<Mutex<Node>>)
    -> Result<HttpResponse, Box<dyn Error>> {
    let mut body: web::BytesMut = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        body.extend_from_slice(&chunk);
    }
    let peer = serde_json::from_slice::<Peer>(&body)?;
    println!("{:#?}", peer);

    let mut node = data.lock().unwrap();
    if !node.peers.contains(&peer) {
        node.peers.push(peer);
    }

    Ok(HttpResponse::Ok().json(&node.peers))
}

#[post("/send")]
async fn send(mut payload: web::Payload, data: web::Data<Mutex<Node>>)
    -> Result<HttpResponse, Box<dyn Error>> {
    let mut body: Vec<u8> = Vec::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        body.extend_from_slice(&chunk);
    }
    let mut node = data.lock().unwrap();
    node.latest_sel = body;
    println!("{:#?}", str::from_utf8(&node.latest_sel).unwrap());

    Ok(HttpResponse::Ok().json(&node.hostname))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut node = init_node(true)?;
    println!("{:#?}", node);

    let data = web::Data::new(Mutex::new(node));

    HttpServer::new(move || App::new()
        .app_data(data.clone())
        .service(latest)
        .service(peers)
        .service(send)
    ).bind(("127.0.0.1", PORT))?.run().await;

    Ok(())
}
