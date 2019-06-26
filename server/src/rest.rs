use std::net::SocketAddr;
use hyper::service::service_fn;
use hyper::{Body, Server, Request, Response, Client, Method, StatusCode};
use hyper::client::HttpConnector;
use futures::{future, Future};
use http::header::{AUTHORIZATION,WWW_AUTHENTICATE};
use std::fs;
use url::Url;
use log::error;
use std::fs::OpenOptions;
use std::io::Write;

type GenericError = Box<dyn std::error::Error + Send + Sync>;
type ResponseFuture = Box<dyn Future<Item=Response<Body>, Error=GenericError> + Send>;

fn unauthorized() -> ResponseFuture{
    Box::new(future::ok(
        Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .header(WWW_AUTHENTICATE, "Basic realm=\"User Visible Realm\"")
            .body(Body::from(b"Unauthorized".as_ref()))
            .unwrap()
        )
    )
}

fn failure(message: &str) -> ResponseFuture{
    Box::new(future::ok(
        Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::from(format!("Server error: {}", message)))
            .unwrap()
        )
    )
}

fn success(message: &str) -> ResponseFuture{
    Box::new(future::ok(
        Response::builder()
            .status(StatusCode::OK)
            .body(Body::from(format!("Ok {}", message)))
            .unwrap()
        )
    )
}

fn line_in_file(line: &String, filename: &String) -> bool{
    let file = match fs::read_to_string(filename){
        Ok(i) => i,
        Err(_) => return false,
    };
    for i in file.split("\n"){
        if i.is_empty() || i.trim_start().starts_with("#"){
            continue;
        }
        if &i == line{
            return true;
        }
    }
    return false;
}

fn routes(req: Request<Body>, _client: &Client<HttpConnector>, passwordsfile: String, whitelist: String, blacklist: String) -> ResponseFuture {

    let auth: &str = match req.headers().get(AUTHORIZATION){
        Some(i) => {
            match i.to_str(){
                Ok(i) => i,
                Err(_) => return unauthorized(),
            }
        },
        None => return unauthorized(),
    };
    let passwordparts: Vec<&str> = auth.split("Basic ").collect();
    let encodedpassword = match passwordparts.get(1){
        Some(i) => i,
        None => return unauthorized(),
    };

    let decodedvec =match base64::decode(encodedpassword){
        Ok(i) => i,
        Err(_) => return unauthorized(),
    };

    let decodedpassword =  match String::from_utf8(decodedvec){
        Ok(i) => i,
        Err(_) => return unauthorized(),
    };

    if !line_in_file(&decodedpassword, &passwordsfile){
        return unauthorized();
    }

    let uri_string = String::from("https://example.com") + req.uri().to_string().as_str();
    let request_url = match Url::parse(&uri_string){
        Ok(i) => i,
        Err(i) => {
            error!("An error occurred during the url parsing: {}",i);
            return failure("");
        }
    };
    let mut params = request_url.query_pairs();

    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") => {
            let body = Body::from(b"Collabdraw api".as_ref());
            Box::new(future::ok(Response::new(body)))
        },

        (&Method::GET, "/whitelist/add") => {
            if params.count() == 1{
                let param = match params.next(){
                    Some(i) => i,
                    None => return failure("malformed query parameter"),
                };
                if param.0.to_owned() == "ip"{
                    let iptoadd = String::from(param.1);
                    if line_in_file(&iptoadd, &whitelist){
                        return failure("Ip already in whitelist");
                    }

                    let mut file = match OpenOptions::new()
                        .append(true)
                        .open(whitelist){
                        Ok(i) => i,
                        Err(_) => return failure("couldn't open file"),
                    };

                    match file.write(iptoadd.as_bytes()){
                        Ok(i) => i,
                        Err(_) => return failure("write failure"),
                    };
                    return success("");
                }else{
                    return failure("Please give ?ip=xx.xx.xx.xx");
                }
            }else{
                return failure("Please give ?ip=xx.xx.xx.xx");
            }
        },

        (&Method::GET, "/whitelist/remove") | (&Method::GET, "/whitelist/delete") => {
            if params.count() == 1{
                let param = match params.next(){
                    Some(i) => i,
                    None => return failure("malformed query parameter"),
                };
                if param.0.to_owned() == "ip"{
                    let iptodel = String::from(param.1);
                    if !line_in_file(&iptodel, &whitelist) {
                        return failure("Ip not in whitelist");
                    }

                    let file = match fs::read_to_string(&whitelist){
                        Ok(i) => i,
                        Err(_) => return failure("couldn't open file"),
                    };

                    let removed = file.replace( &*iptodel, "");

                    let mut file = match OpenOptions::new()
                        .write(true)
                        .open(whitelist){
                        Ok(i) => i,
                        Err(_) => return failure("couldn't open file"),
                    };

                    match file.set_len(0){
                        Ok(i) => i,
                        Err(_) => return failure("write failure"),
                    };
                    match file.write(removed.as_bytes()){
                        Ok(i) => i,
                        Err(_) => return failure("write failure"),
                    };

                    return success("");
                }else{
                    return failure("Please give ?ip=xx.xx.xx.xx");
                }
            }else{
                return failure("Please give ?ip=xx.xx.xx.xx");
            }

        },
        (&Method::GET, "/whitelist/list") => {

            let whitelistcontents = match fs::read_to_string(whitelist){
              Ok(i) => i,
              Err(_) => return failure(""),
            };
            let body = Body::from(whitelistcontents);
            Box::new(future::ok(Response::new(body)))
        },

        _ => {
            // Return 404 not found response.
            let body = Body::from(b"Page not found".as_ref());
            Box::new(future::ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(body)
                .unwrap()))
        }
    }
}

pub fn main(address: SocketAddr, passwords: String, whitelist: String, blacklist: String){

    hyper::rt::run(future::lazy(move || {

        // Share a `Client` with all `Service`s
        let client = Client::new();

        let new_service = move || {
            // Move a clone of `client` into the `service_fn`.
            let client = client.clone();
            let pfilename = passwords.clone();
            let wfilename = whitelist.clone();
            let bfilename = blacklist.clone();


            service_fn(move |req| {
                routes(req, &client, pfilename.clone(), wfilename.clone(), bfilename.clone())
            })
        };

        let server = Server::bind(&address)
            .serve(new_service)
            .map_err(|e| eprintln!("server error: {}", e));

        println!("Listening on http://{}", address);

        server
    }));

}