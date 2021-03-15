use futures_util::stream::{self};
use tonic::Request;
// use tonic::IntoStreamingRequest;

use tokio::task;

use std::sync::Arc;
use std::sync::atomic::*;
use std::{thread, time, io};

use common::ss_user_update;
#[allow(unused_imports)]
use common::ss_response;
use common::SsRequest;
use common::SsUserUpdate;
use common::SsSubscriptionUpdate;
use common::session_client::SessionClient;

fn get_token() -> String {
    String::from("token")
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cert = include_str!("../client.pem");
    let key = include_str!("../client.key");
    let id = tonic::transport::Identity::from_pem(cert.as_bytes(),key.as_bytes());
    let s = include_str!("../my_ca.pem");
    let ca = tonic::transport::Certificate::from_pem(s.as_bytes());
    let _tls = tonic::transport::ClientTlsConfig::new().domain_name("localhost").identity(id).ca_certificate(ca);

    let channel = tonic::transport::Channel::from_static("http://localhost:54321")
        // ._tls_config(tls).unwrap()
        .connect()
        .await?;
    let token = get_token();
    let channel_copy = channel.clone();

    let mut client = SessionClient::with_interceptor(channel, move |mut req: Request<()>| {
        req.metadata_mut().insert(
            "authorization",
            tonic::metadata::MetadataValue::from_str(&token).unwrap(),
        );
        Ok(req)
    });

    let request = tonic::Request::new(SsRequest {
        user_id: 123,
        msg_timestamp: 9878374,
        user_hash: String::from("blah-blah-blah"),
        start_point: Option::from( SsUserUpdate {
            latitude: 0.0,
            longitude: 0.0,
            location_timestamp: 23984794,
            status: ss_user_update::Status::Passive as i32,
        } ),
        opt_is_continuation: Option::default(),
    });

    println!("For exit press 1<Enter>");
    println!("Trying to open a session...");

    let response = client.open(request).await?.into_inner();
    let err_result = response.result as i32;
    if err_result != ss_response::Result::Success as i32 {
        println!("Session was not established! {}", err_result);
        panic!("Fatal error!");
    }
    println!("RESPONSE: session ID = {}", response.session_id);

    let mutate = Arc::new(AtomicBool::new(true));
    let flag_clone = mutate.clone();
    // let _another_client: session_client::SessionClient;
    let async_client_fut = task::spawn(update_stream(channel_copy, flag_clone));

    println!("Waining for a console input...");
    let mut input = String::new();
    while let _res = io::stdin().read_line(&mut input) {
        // io::stdin().read_line(&mut input)?;
        let mut _parse_res = match input.trim().parse::<u32>() {
            Ok(num) => {
                match num {
                    1 => {
                        println!("Breaking console reading loop!");
                        break;
                    }
                    _ => println!("Enter 1 for exit. Read {}", input),
                };
            }
            Err(_error) => {
                println!("Enter 1 for exit. Read {}", input);
                println!("Error: {}", _error);
                input.clear();
                continue;
            }
        };
    }
    mutate.store(false, Ordering::Relaxed);

    match async_client_fut.await {
        Ok(_) => println!("Async processing..."),
        Err(_) => println!("Failed async client creation!")
    };

    let request_close = tonic::Request::new(SsRequest {
        user_id: 123,
        msg_timestamp: 9878374,
        user_hash: String::from("blah-blah-blah"),
        start_point: Option::from( SsUserUpdate {
            latitude: 0.0,
            longitude: 0.0,
            location_timestamp: 23984794,
            status: ss_user_update::Status::Passive as i32,
        } ),
        opt_is_continuation: Option::default(),
    });

    let _close_res = client.close(request_close).await?.into_inner();

    Ok(())
}

// #############################################################################
//
async fn update_stream(channel: tonic::transport::Channel, running: Arc<AtomicBool>) 
    -> Result<(),()> 
{
    let token = get_token();
    let mut client = SessionClient::with_interceptor(channel, move |mut req: Request<()>| {
        req.metadata_mut().insert(
            "authorization",
            tonic::metadata::MetadataValue::from_str(&token).unwrap(),
        );
        Ok(req)
    });
    
    let mut results = Vec::<SsSubscriptionUpdate>::new();

    let mut iter_count: u16 = 0;
    let mut session_is_active = true;
    let hundred_millis = time::Duration::from_millis(1000);

    println!("Starting async sending loop");
    while session_is_active == true && running.load(Ordering::SeqCst) {
        let mut sec_utime: u64 = 0;
        match time::SystemTime::now().duration_since(time::SystemTime::UNIX_EPOCH) {
            Ok(n) => sec_utime = n.as_secs(),
            Err(_) => panic!("SystemTime before UNIX EPOCH!"),
        }
        let client_update = vec!( SsUserUpdate {
            latitude: 1.0,
            longitude: 2.0,
            location_timestamp: sec_utime,
            status: ss_user_update::Status::Passive as i32,
        } );
        let update_result = client.update(stream::iter(client_update)).await;

        match update_result {
            Ok(stream_resp) => {
                let subscription_update = stream_resp.into_inner().message().await;
                match subscription_update {
                    Ok( Some(stream_msg) ) => {
                        println!("Rcvd: {:?}", stream_msg);
                        results.push(stream_msg);
                    }
                    Ok( None ) => println!("none"),
                    Err(_) => println!("err")
                }
            }
            Err(_)  => break,
        }

        thread::sleep(hundred_millis);        

        println!("Awaiting session to close... {}", iter_count);
        iter_count += 1;
        if iter_count > 300 {
            session_is_active = false;
        }
    }

    Ok(())
}

// ###################################################################################